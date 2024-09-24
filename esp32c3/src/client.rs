use core::{fmt::Write, str::from_utf8};
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Stack,
};
use esp_hal::peripherals::RSA;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use heapless::String;
use reqwless::{client::HttpClient, request::Method, Certificates};
use static_cell::{ConstStaticCell, StaticCell};

static TCP_CLIENT_STATE: ConstStaticCell<TcpClientState<1, 4096, 4096>> =
    ConstStaticCell::new(TcpClientState::<1, 4096, 4096>::new());

static TCP_CLIENT: StaticCell<TcpClient<WifiDevice<'static, WifiStaDevice>, 1, 4096, 4096>> =
    StaticCell::new();

static DNS_SOCKET: StaticCell<DnsSocket<WifiDevice<WifiStaDevice>>> = StaticCell::new();

static mut READ_RECORD_BUFFER: [u8; 4096] = [0_u8; 4096];
static mut WRITE_RECORD_BUFFER: [u8; 4096] = [0_u8; 4096];

static RSA: StaticCell<RSA> = StaticCell::new();

pub struct Client<'a, const N: usize = 16512> {
    buffer: [u8; N],
    // stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    inner: HttpClient<
        'a,
        TcpClient<'a, WifiDevice<'static, WifiStaDevice>, 1, 4096, 4096>,
        DnsSocket<'a, WifiDevice<'static, WifiStaDevice>>,
    >,
    // read_record_buffer: [u8; 16640],
    // write_record_buffer: [u8; 16640],
}

impl<'a, const N: usize> Client<'a, N> {
    pub fn new(stack: &'static Stack<WifiDevice<'a, WifiStaDevice>>, rsa: RSA) -> Client<'a, N> {
        let state = TCP_CLIENT_STATE.take();
        let tcp_client = &*TCP_CLIENT.init(TcpClient::new(stack, state));
        let dns_socket = &*DNS_SOCKET.init(DnsSocket::new(stack));
        // TODO figure out safe way for read and write buffers? And maybe find out what tls is :)
        let rsa = RSA.init(rsa);
        let tls_config = reqwless::client::TlsConfig::new(
            unsafe { &mut *core::ptr::addr_of_mut!(READ_RECORD_BUFFER) }, // Safety: No one else is going to access the static variable
            unsafe { &mut *core::ptr::addr_of_mut!(WRITE_RECORD_BUFFER) }, // Safety: No one else is going to access the static variable
            reqwless::TlsVersion::Tls1_3,
            Certificates {
                ca_chain: reqwless::X509::pem(
                    concat!(include_str!("../certs/entsoe.eu.pem"), "\0").as_bytes(),
                )
                .ok(),
                ..Default::default()
            },
            Some(rsa),
        );

        let client = reqwless::client::HttpClient::new_with_tls(tcp_client, dns_socket, tls_config);

        Self {
            buffer: [0; N],
            inner: client,
        }
    }
}

impl Client<'_> {
    pub async fn fetch_local_time(&mut self, tz: chrono_tz::Tz) -> &[u8] {
        let mut url: String<128> = String::new();
        write!(
            &mut url,
            "http://worldtimeapi.org/api/timezone/{}",
            tz.name()
        )
        .unwrap();
        self.get_request(url.as_str()).await
    }

    pub async fn fetch_todays_spot_price(&mut self, token: &str) -> &str {
        let domain = "10YFI-1--------U"; // Todo make domain configurable?

        let mut url = String::<256>::new();
        let period_start = "202408262300";
        let period_end = "202408272200";
        write!(&mut url, "https://web-api.tp.entsoe.eu/api?securityToken={}&DocumentType=A44&PeriodStart={}&PeriodEnd={}&In_Domain={domain}&Out_Domain={domain}",
              token, period_start, period_end).expect("URL buffer probably too small");

        self.get_request_str(&url).await
    }

    pub async fn get_request_str(&mut self, url: &str) -> &str {
        let mut request = self.inner.request(Method::GET, url).await.unwrap();

        let response = request.send(&mut self.buffer).await.unwrap();
        let body = from_utf8(response.body().read_to_end().await.unwrap()).unwrap();

        body
    }

    pub async fn test_connection(&mut self) -> &[u8] {
        self.get_request("https://catfact.ninja/fact").await
    }

    async fn get_request(&mut self, url: &str) -> &[u8] {
        let mut request = self.inner.request(Method::GET, url).await.unwrap();
        let response = request.send(&mut self.buffer).await.unwrap();

        response.body().read_to_end().await.unwrap()
    }
}

// pub trait BuildState {}

// pub struct NotStarted {}
// pub struct Started {}
// pub struct Ready<'a> {
//     client: HttpClient<
//         'a,
//         TcpClient<'a, WifiDevice<'static, WifiStaDevice>, 1, 4096, 4096>,
//         DnsSocket<'a, WifiDevice<'static, WifiStaDevice>>,
//     >,
// }

// impl BuildState for NotStarted {}
// impl BuildState for Started {}
// impl BuildState for Ready<'_> {}
