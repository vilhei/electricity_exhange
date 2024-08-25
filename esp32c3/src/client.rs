use core::{fmt::Write, str::from_utf8};
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Stack,
};
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use heapless::String;
use reqwless::{
    client::HttpClient,
    headers::ContentType,
    request::{Method, RequestBuilder},
};
use static_cell::{ConstStaticCell, StaticCell};

static TCP_CLIENT_STATE: ConstStaticCell<TcpClientState<1, 4096, 4096>> =
    ConstStaticCell::new(TcpClientState::<1, 4096, 4096>::new());

static TCP_CLIENT: StaticCell<TcpClient<WifiDevice<'static, WifiStaDevice>, 1, 4096, 4096>> =
    StaticCell::new();

static DNS_SOCKET: StaticCell<DnsSocket<WifiDevice<WifiStaDevice>>> = StaticCell::new();

static mut READ_RECORD_BUFFER: [u8; 16640] = [0_u8; 16640];
static mut WRITE_RECORD_BUFFER: [u8; 16640] = [0_u8; 16640];

pub struct Client<'a, const N: usize = 4096> {
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

impl<'a> Client<'a> {
    pub fn new(stack: &'static Stack<WifiDevice<'a, WifiStaDevice>>) -> Client<'a> {
        let state = TCP_CLIENT_STATE.take();
        let tcp_client = &*TCP_CLIENT.init(TcpClient::new(stack, state));
        let dns_socket = &*DNS_SOCKET.init(DnsSocket::new(stack));
        // let mut read_record_buffer = [0_u8; 16640];
        // let mut write_record_buffer = [0_u8; 16640];

        let tls_config = reqwless::client::TlsConfig::new(
            0,
            unsafe { &mut *core::ptr::addr_of_mut!(READ_RECORD_BUFFER) },
            unsafe { &mut *core::ptr::addr_of_mut!(WRITE_RECORD_BUFFER) },
            reqwless::client::TlsVerify::None,
        );

        let client = reqwless::client::HttpClient::new_with_tls(tcp_client, dns_socket, tls_config);

        Self {
            // buffer: todo!(),
            buffer: [0; 4096],
            // stack,
            inner: client, // state: Ready { client },
                           // read_record_buffer: [0_u8; 16640],
                           // write_record_buffer: [0_u8; 16640],
        }
    }
}

impl Client<'_> {
    pub async fn fetch_local_time(&mut self, tz: chrono_tz::Tz) -> &[u8] {
        let mut url: String<128> = String::new();
        write!(
            &mut url,
            "https://worldtimeapi.org/api/timezone/{}",
            tz.name()
        )
        .unwrap();
        // dbg!(&url);
        self.get_request(url.as_str()).await
    }


    pub async fn get_request_str(&mut self, url: &str) -> &str {
        let mut request = self
            .inner
            .request(Method::GET, url)
            .await
            .unwrap()
            .content_type(ContentType::ApplicationJson);

        let response = request.send(&mut self.buffer).await.unwrap();
        let body = from_utf8(response.body().read_to_end().await.unwrap()).unwrap();

        body
    }
    async fn get_request(&mut self, url: &str) -> &[u8] {
        let mut request = self
            .inner
            .request(Method::GET, url)
            .await
            .unwrap()
            .content_type(ContentType::TextPlain);

        let response = request.send(&mut self.buffer).await.unwrap();
        // dbg!(&response.status);

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
