use core::str::from_utf8;

use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Stack,
};
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use reqwless::{client::HttpClient, request::RequestBuilder};
use static_cell::{ConstStaticCell, StaticCell};

static TCP_CLIENT_STATE: ConstStaticCell<TcpClientState<1, 4096, 4096>> =
    ConstStaticCell::new(TcpClientState::<1, 4096, 4096>::new());
static TCP_CLIENT: StaticCell<TcpClient<WifiDevice<'static, WifiStaDevice>, 1, 4096, 4096>> =
    StaticCell::new();
static DNS_SOCKET: StaticCell<DnsSocket<WifiDevice<WifiStaDevice>>> = StaticCell::new();

pub struct Client<S: BuildState, const N: usize = 4096> {
    buffer: [u8; N],
    // stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    state: S,
}

impl<'a> Client<NotStarted> {
    pub fn new(stack: &'static Stack<WifiDevice<WifiStaDevice>>) -> Client<Ready<'a>> {
        let state = TCP_CLIENT_STATE.take();
        let tcp_client = &*TCP_CLIENT.init(TcpClient::new(stack, state));
        let dns_socket = &*DNS_SOCKET.init(DnsSocket::new(stack));
        let client = reqwless::client::HttpClient::new(tcp_client, dns_socket);

        Client::<Ready<'a>> {
            // buffer: todo!(),
            buffer: [0; 4096],
            // stack,
            state: Ready { client },
        }
    }
}

impl Client<Ready<'_>> {
    pub async fn request(&mut self) -> &str {
        let mut request = self
            .state
            .client
            .request(reqwless::request::Method::GET, "http://httpbin.org/get")
            .await
            .unwrap()
            .content_type(reqwless::headers::ContentType::ApplicationJson);

        let response = request.send(&mut self.buffer).await.unwrap();
        let body = from_utf8(response.body().read_to_end().await.unwrap()).unwrap();

        body
    }
}

pub trait BuildState {}

pub struct NotStarted {}
pub struct Started {}
pub struct Ready<'a> {
    client: HttpClient<
        'a,
        TcpClient<'a, WifiDevice<'static, WifiStaDevice>, 1, 4096, 4096>,
        DnsSocket<'a, WifiDevice<'static, WifiStaDevice>>,
    >,
}

impl BuildState for NotStarted {}
impl BuildState for Started {}
impl BuildState for Ready<'_> {}
