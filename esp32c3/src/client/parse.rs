use core::str::FromStr;

use esp_println::println;
use heapless::{String, Vec};
use xmlparser::{Error, Token};

#[derive(Debug)]
pub struct Entsoe<const N: usize = 24> {
    pub mr_id: String<64>,
    pub r#type: String<64>,
    pub time_series: TimeSeries<N>,
}

#[derive(Debug)]
pub struct TimeSeries<const N: usize> {
    pub mrid: u32,
    pub currency_unit: String<16>,
    pub power_unit: String<16>,
    pub period: Period<N>,
}

#[derive(Debug)]
pub struct Period<const N: usize> {
    pub time_interval: TimeInterval,
    pub points: Vec<Point, N>,
}

#[derive(Debug)]
pub struct TimeInterval {
    pub start: String<64>,
    pub end: String<64>,
}

#[derive(Debug)]
pub struct Point {
    pub position: u32,
    pub price: f32,
}

pub fn parse_day_prices(xml: &str) -> Entsoe {
    let mut xml_iter = xmlparser::Tokenizer::from(xml).peekable();
    // let mut mr_id: Option<&str> = None;
    // let mut r#type: Option<&str> = None;
    // let mut it = xml_tokenizer.next();

    let mr_id = get_element_value(&mut xml_iter, "mRID");
    let document_type = get_element_value(&mut xml_iter, "type");
    let timeseries_mr_id = get_element_value(&mut xml_iter, "mRID");
    let currency_unit = get_element_value(&mut xml_iter, "currency_Unit.name");
    let power_unit = get_element_value(&mut xml_iter, "price_Measure_Unit.name");

    skip_until_element(&mut xml_iter, "Period");
    let start_date = get_element_value(&mut xml_iter, "start");
    let end_date = get_element_value(&mut xml_iter, "end");

    let mut points: Vec<Point, 24> = Vec::new();

    get_points(&mut xml_iter, &mut points);

    // let p = get_point(&mut xml_iter);

    println!("{mr_id} - {document_type} - {timeseries_mr_id} - {currency_unit} - {power_unit}\n {start_date} - {end_date}\n{points:#?}");
    let e = Entsoe {
        mr_id: String::from_str(mr_id).unwrap(),
        r#type: String::from_str(document_type).unwrap(),
        time_series: TimeSeries {
            mrid: timeseries_mr_id.parse().unwrap(),
            currency_unit: String::from_str(currency_unit).unwrap(),
            power_unit: String::from_str(power_unit).unwrap(),
            period: Period {
                time_interval: TimeInterval {
                    start: String::from_str(start_date).unwrap(),
                    end: String::from_str(end_date).unwrap(),
                },
                points,
            },
        },
    };
    e
}

/// Reads `buffer.len()` amount of points from the XML into the `buffer`
/// If the xml does not contain correct amount of points results in ????
/// Decide whether to panic of return less points or?
fn get_points<'a, I, const N: usize>(xml_iter: &mut I, buffer: &mut Vec<Point, N>)
where
    I: Iterator<Item = Result<Token<'a>, Error>>,
{
    buffer.clear();
    for _ in 0..N {
        buffer.push(get_point(xml_iter));
    }
}

fn get_point<'a, I>(xml_iter: &mut I) -> Point
where
    I: Iterator<Item = Result<Token<'a>, Error>>,
{
    skip_until_element(xml_iter, "Point");
    let position = get_element_value(xml_iter, "position");
    let price = get_element_value(xml_iter, "price.amount");

    Point {
        position: position.parse().unwrap(),
        price: price.parse().unwrap(),
    }
}

fn get_element_value<'a, I>(xml_iter: &mut I, element_name: &'a str) -> &'a str
where
    I: Iterator<Item = Result<Token<'a>, Error>>,
{
    skip_until_element(xml_iter, element_name);
    get_next_value(xml_iter)
}

fn skip_until_element<'a, I>(xml_iter: &mut I, element_name: &'a str)
where
    I: Iterator<Item = Result<Token<'a>, Error>>,
{
    loop {
        let token = xml_iter.next().expect("iterator ended").unwrap();

        if let Token::ElementStart { local, .. } = token {
            if local.as_str() == element_name {
                return;
            }
        }
    }
}

/// Advances iterator until it gets next [xmlparser::Token::Text] and returns the content.
fn get_next_value<'a, I>(xml_tokenizer: &mut I) -> &'a str
where
    I: Iterator<Item = Result<Token<'a>, Error>>,
{
    loop {
        let token = xml_tokenizer
            .next()
            .expect("No value following elementstart")
            .unwrap();

        if let Token::Text { text } = token {
            return text.as_str();
        }
    }
}
