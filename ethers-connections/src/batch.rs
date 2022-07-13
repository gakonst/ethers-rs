use std::vec;

use serde::Deserialize;
use serde_json::value::RawValue;

use crate::{
    call::{CallParams, RpcCall},
    connection::ConnectionError,
    Connection, DynFuture,
};

const PANIC_MSG: &str = "failed to serialize batch request as JSON";

#[derive(Debug)]
pub enum BatchError {
    Connection(ConnectionError),
    IncompleteBatch,
}

impl From<ConnectionError> for BatchError {
    fn from(err: ConnectionError) -> Self {
        Self::Connection(err)
    }
}
pub trait BatchCall {
    type Output: Sized;

    fn send_batch<C: Connection>(
        self,
        connection: &C,
    ) -> DynFuture<'_, Result<Self::Output, BatchError>>;
}

impl<CONN, T: for<'de> Deserialize<'de>> BatchCall for Vec<RpcCall<CONN, T>> {
    type Output = Vec<T>;

    fn send_batch<C: Connection>(
        self,
        connection: &C,
    ) -> DynFuture<'_, Result<Self::Output, BatchError>> {
        let (ids, requests): (Vec<_>, Vec<_>) = self
            .into_iter()
            .map(|call| {
                let params = call.to_params();
                (params.id, params.request)
            })
            .unzip();
        let request = serde_json::value::to_raw_value(&requests).expect(PANIC_MSG);

        Box::pin(async move {
            let responses =
                connection.send_raw_batch_request(ids.into_boxed_slice(), request).await?;

            let mut result = Vec::with_capacity(responses.len());
            for res in responses {
                let raw = res?;
                let response = serde_json::from_str(raw.get())
                    .map_err(|source| ConnectionError::json(raw.get(), source))?;

                result.push(response);
            }

            Ok(result)
        })
    }
}

pub struct BatchResponse {
    response: vec::IntoIter<Result<Box<RawValue>, ConnectionError>>,
}

impl BatchCall for Vec<CallParams> {
    type Output = BatchResponse;

    fn send_batch<C: Connection>(
        self,
        connection: &C,
    ) -> DynFuture<'_, Result<Self::Output, BatchError>> {
        let (ids, request): (Vec<_>, Vec<_>) =
            self.into_iter().map(|params| (params.id, params.request)).unzip();
        let request = serde_json::value::to_raw_value(&request).expect("TODO");

        Box::pin(async move {
            let response =
                connection.send_raw_batch_request(ids.into_boxed_slice(), request).await?;
            Ok(BatchResponse { response: Vec::from(response).into_iter() })
        })
    }
}

impl BatchResponse {
    pub fn parse_next<R: for<'de> Deserialize<'de>>(&mut self) -> Result<R, BatchError> {
        let next = self.response.next().ok_or(BatchError::IncompleteBatch)??;
        let res = serde_json::from_str(next.get())
            .map_err(|source| ConnectionError::json(next.get(), source))?;

        Ok(res)
    }
}

macro_rules! impl_batch_call {
    ($count:expr, $($ty:ident),*) => {
        #[allow(unused_parens)]
        impl<CONN, $($ty),*> BatchCall for ($(RpcCall<CONN, $ty>),*)
        where
            $($ty: for <'de> Deserialize<'de>),*
        {
            type Output = ($($ty),*);

            fn send_batch<C: Connection>(
                self,
                connection: &C,
            ) -> DynFuture<'_, Result<Self::Output, BatchError>> {
                // destruct tuple to extract individual calls
                #[allow(non_snake_case)]
                let (
                    $(
                        $ty
                    ),*
                ) = self;

                // destruct tuple to extract individual calls
                #[allow(non_snake_case)]
                let (
                    $(
                        $ty
                    ),*
                ) =  ($($ty.to_params()),*);

                // extract all request ids in the same order as the tuple
                let ids = Box::new([
                    $(
                        $ty.id
                    ),*
                ]);

                // serialize the batch request
                let request = serde_json::value::to_raw_value(&[
                    $(
                        $ty.request
                    ),*
                ])
                    .expect(PANIC_MSG);

                Box::pin(async move {
                    let responses = connection.send_raw_batch_request(ids, request).await?;
                    #[allow(non_snake_case)]
                    match <[Result<Box<RawValue>, ConnectionError>; $count]>::try_from(responses) {
                        Ok([$($ty),*]) => {
                            Ok((
                                $(
                                    {
                                        let raw = $ty?;
                                        serde_json::from_str(raw.get())
                                            .map_err(|source| ConnectionError::json(
                                                raw.get(),
                                                source
                                            ))?
                                    }
                                ),*
                            ))
                        },
                        _ => unreachable!(),
                    }
                })
            }
        }
    };
}

impl_batch_call!(1, T1);
impl_batch_call!(2, T1, T2);
impl_batch_call!(3, T1, T2, T3);
impl_batch_call!(4, T1, T2, T3, T4);
impl_batch_call!(5, T1, T2, T3, T4, T5);
impl_batch_call!(6, T1, T2, T3, T4, T5, T6);
impl_batch_call!(7, T1, T2, T3, T4, T5, T6, T7);
impl_batch_call!(8, T1, T2, T3, T4, T5, T6, T7, T8);
impl_batch_call!(9, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_batch_call!(10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_batch_call!(11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_batch_call!(12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_batch_call!(13, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_batch_call!(14, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_batch_call!(15, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_batch_call!(16, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
