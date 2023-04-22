use crate::{utils::PinBoxFut, JsonRpcClient, Middleware, Provider, ProviderError};
use ethers_core::types::{Filter, Log, U64};
use futures_core::stream::Stream;
use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
};
use thiserror::Error;

/// A log query provides streaming access to historical logs via a paginated
/// request. For streaming access to future logs, use [`Middleware::watch`] or
/// [`Middleware::subscribe_logs`]
pub struct LogQuery<'a, P> {
    provider: &'a Provider<P>,
    filter: Filter,
    from_block: Option<U64>,
    page_size: u64,
    current_logs: VecDeque<Log>,
    last_block: Option<U64>,
    state: LogQueryState<'a>,
}

enum LogQueryState<'a> {
    Initial,
    LoadLastBlock(PinBoxFut<'a, U64>),
    LoadLogs(PinBoxFut<'a, Vec<Log>>),
    Consume,
}

impl<'a, P> LogQuery<'a, P>
where
    P: JsonRpcClient,
{
    /// Instantiate a new `LogQuery`
    pub fn new(provider: &'a Provider<P>, filter: &Filter) -> Self {
        Self {
            provider,
            filter: filter.clone(),
            from_block: filter.get_from_block(),
            page_size: 10000,
            current_logs: VecDeque::new(),
            last_block: None,
            state: LogQueryState::Initial,
        }
    }

    /// set page size for pagination
    pub fn with_page_size(mut self, page_size: u64) -> Self {
        self.page_size = page_size;
        self
    }
}

macro_rules! rewake_with_new_state {
    ($ctx:ident, $this:ident, $new_state:expr) => {
        $this.state = $new_state;
        $ctx.waker().wake_by_ref();
        return Poll::Pending
    };
}

/// Errors while querying for logs
#[derive(Error, Debug)]
pub enum LogQueryError<E> {
    /// Error loading latest block
    #[error(transparent)]
    LoadLastBlockError(E),
    /// Error loading logs from block range
    #[error(transparent)]
    LoadLogsError(E),
}

impl<'a, P> Stream for LogQuery<'a, P>
where
    P: JsonRpcClient,
{
    type Item = Result<Log, LogQueryError<ProviderError>>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.state {
            LogQueryState::Initial => {
                if !self.filter.is_paginatable() {
                    // if not paginatable, load logs and consume
                    let filter = self.filter.clone();
                    let provider = self.provider;
                    #[allow(clippy::redundant_async_block)]
                    let fut = Box::pin(async move { provider.get_logs(&filter).await });
                    rewake_with_new_state!(ctx, self, LogQueryState::LoadLogs(fut));
                } else {
                    // if paginatable, load last block
                    let fut = self.provider.get_block_number();
                    rewake_with_new_state!(ctx, self, LogQueryState::LoadLastBlock(fut));
                }
            }
            LogQueryState::LoadLastBlock(fut) => {
                match futures_util::ready!(fut.as_mut().poll(ctx)) {
                    Ok(last_block) => {
                        self.last_block = Some(last_block);

                        // this is okay because we will only enter this state when the filter is
                        // paginatable i.e. from block is set
                        let from_block = self.filter.get_from_block().unwrap();
                        let to_block = from_block + self.page_size;
                        self.from_block = Some(to_block + 1);

                        let filter = self.filter.clone().from_block(from_block).to_block(to_block);
                        let provider = self.provider;
                        // load first page of logs
                        #[allow(clippy::redundant_async_block)]
                        let fut = Box::pin(async move { provider.get_logs(&filter).await });
                        rewake_with_new_state!(ctx, self, LogQueryState::LoadLogs(fut));
                    }
                    Err(err) => Poll::Ready(Some(Err(LogQueryError::LoadLastBlockError(err)))),
                }
            }
            LogQueryState::LoadLogs(fut) => match futures_util::ready!(fut.as_mut().poll(ctx)) {
                Ok(logs) => {
                    self.current_logs = VecDeque::from(logs);
                    rewake_with_new_state!(ctx, self, LogQueryState::Consume);
                }
                Err(err) => Poll::Ready(Some(Err(LogQueryError::LoadLogsError(err)))),
            },
            LogQueryState::Consume => {
                let log = self.current_logs.pop_front();
                if log.is_none() {
                    // consumed all the logs
                    if !self.filter.is_paginatable() {
                        Poll::Ready(None)
                    } else {
                        // load new logs if there are still more pages to go through
                        // can safely assume this will always be set in this state
                        let from_block = self.from_block.unwrap();
                        let to_block = from_block + self.page_size;

                        // no more pages to load, and everything is consumed
                        // can safely assume this will always be set in this state
                        if from_block > self.last_block.unwrap() {
                            return Poll::Ready(None)
                        }
                        // load next page
                        self.from_block = Some(to_block + 1);

                        let filter = self.filter.clone().from_block(from_block).to_block(to_block);
                        let provider = self.provider;
                        #[allow(clippy::redundant_async_block)]
                        let fut = Box::pin(async move { provider.get_logs(&filter).await });
                        rewake_with_new_state!(ctx, self, LogQueryState::LoadLogs(fut));
                    }
                } else {
                    Poll::Ready(log.map(Ok))
                }
            }
        }
    }
}
