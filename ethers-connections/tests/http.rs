use tokio::runtime::Builder;

use ethers_core::utils::Geth;

use ethers_connections::{connection::http::Http, Provider};

#[test]
fn http_batch() {
    use ethers_core::types::Address;

    let geth = Geth::new().port(8545u16).block_time(5u64).spawn();

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let connection =
            Http::new("http://127.0.0.1:8545").expect("failed to build HTTP connection");
        let provider = Provider { connection };

        let address1 = Address::from_low_u64_be(1);
        let address2 = Address::from_low_u64_be(2);
        let address3 = Address::from_low_u64_be(3);
        let address4 = Address::from_low_u64_be(4);
        let a = provider.get_balance(&address1, "latest".into());
        let b = provider.get_balance(&address2, "latest".into());
        let c = provider.get_balance(&address3, "latest".into());
        let d = provider.get_balance(&address4, "latest".into());

        let (a, b, c, d) = provider.send_batch_request((a, b, c, d)).await.unwrap();

        assert_eq!(a, 1u64.into());
        assert_eq!(b, 1u64.into());
        assert_eq!(c, 1u64.into());
        assert_eq!(d, 1u64.into());
    });

    drop(geth);
}
