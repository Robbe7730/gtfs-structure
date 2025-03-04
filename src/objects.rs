use core::fmt::Formatter;
use serde::de::MapAccess;
use serde::de::Visitor;
use crate::Gtfs;
use chrono::{Datelike, NaiveDate, Weekday};
use rgb::RGB8;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::fmt;
use std::sync::Arc;

pub trait Id {
    fn id(&self) -> &str;
}

pub trait Type {
    fn object_type(&self) -> ObjectType;
}

pub trait Translatable {
    fn translate(&self, gtfs: &Gtfs, language: &str) -> Self;
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GtfsTranslation {
    pub table_name: String,
    pub field_name: String,
    pub language: String,
    pub translation: String,
    pub record_id: Option<String>,
    pub record_sub_id: Option<String>,
    pub field_value: Option<String>,
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NmbsTranslation {
    pub trans_id: String,
    pub lang: String,
    pub translation: String
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum Translation {
    Gtfs(GtfsTranslation),
    Nmbs(NmbsTranslation),
}

impl<'de> Deserialize<'de> for Translation {
    fn deserialize<D>(deserializer: D) -> Result<Translation, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            // GTFS/shared values
            TableName,
            FieldName,
            Language,
            Translation,
            RecordId,
            RecordSubId,
            FieldValue,
            // NMBS values
            TransId,
            Lang,
        }

        struct TranslationVisitor;

        impl<'de> Visitor<'de> for TranslationVisitor {
            type Value = Translation;

            fn visit_map<V>(self, mut map: V) -> Result<Translation, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut maybe_table_name = None;
                let mut maybe_field_name = None;
                let mut maybe_language = None;
                let mut maybe_translation = None;
                let mut maybe_record_id = None;
                let mut maybe_record_sub_id = None;
                let mut maybe_field_value = None;
                let mut maybe_trans_id = None;
                let mut maybe_lang = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::TableName => {
                            maybe_table_name = Some(map.next_value()?);
                        }
                        Field::FieldName => {
                            maybe_field_name = Some(map.next_value()?);
                        }
                        Field::Language => {
                            maybe_language = Some(map.next_value()?);
                        }
                        Field::Translation => {
                            maybe_translation = Some(map.next_value()?);
                        }
                        Field::RecordId => {
                            maybe_record_id = Some(map.next_value()?);
                        }
                        Field::RecordSubId => {
                            maybe_record_sub_id = Some(map.next_value()?);
                        }
                        Field::FieldValue => {
                            maybe_field_value = Some(map.next_value()?);
                        }
                        Field::TransId => {
                            maybe_trans_id = Some(map.next_value()?);
                        }
                        Field::Lang => {
                            maybe_lang = Some(map.next_value()?);
                        }
                    }
                }

                if maybe_lang.is_some() && maybe_trans_id.is_some() && maybe_translation.is_some() {
                    Ok(Translation::Nmbs(NmbsTranslation {
                        lang: maybe_lang.unwrap(),
                        trans_id: maybe_trans_id.unwrap(),
                        translation: maybe_translation.unwrap()
                    }))
                } else {
                    Ok(Translation::Gtfs(GtfsTranslation {
                        table_name: maybe_table_name.unwrap(),
                        field_name: maybe_field_name.unwrap(),
                        language: maybe_language.unwrap(),
                        translation: maybe_translation.unwrap(),
                        record_id: maybe_record_id.unwrap(),
                        record_sub_id: maybe_record_sub_id.unwrap(),
                        field_value: maybe_field_value.unwrap(),
                    }))
                }
            }
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("struct Translation")
            }
        }

        const FIELDS: &'static [&'static str] = &["table_name", "field_name", "language", "translation", "record_id", "record_sub_id", "field_value", "trans_id", "lang"];
        deserializer.deserialize_struct("Duration", FIELDS, TranslationVisitor)
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct TranslationByIdKey {
    pub table_name: String,
    pub field_name: String,
    pub language: String,
    pub record_id: String,
    pub record_sub_id: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct TranslationByValueKey {
    pub table_name: String,
    pub field_name: String,
    pub language: String,
    pub field_value: String,
}

#[derive(Debug, Serialize, Eq, PartialEq, Hash, Clone)]
pub enum ObjectType {
    Agency,
    Stop,
    Route,
    Trip,
    Calendar,
    Shape,
    Fare,
    StopTime,
    FeedInfo,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub enum LocationType {
    StopPoint = 0,
    StopArea = 1,
    StationEntrance = 2,
    GenericNode = 3,
    BoardingArea = 4,
}

impl<'de> Deserialize<'de> for LocationType {
    fn deserialize<D>(deserializer: D) -> Result<LocationType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "1" => LocationType::StopArea,
            "2" => LocationType::StationEntrance,
            "3" => LocationType::GenericNode,
            "4" => LocationType::BoardingArea,
            _ => LocationType::StopPoint,
        })
    }
}

impl Default for LocationType {
    fn default() -> LocationType {
        LocationType::StopPoint
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RouteType {
    Tramway,
    Subway,
    Rail,
    Bus,
    Ferry,
    CableCar,
    Gondola,
    Funicular,
    // extended GTFS (https://developers.google.com/transit/gtfs/reference/extended-route-types)
    Coach,
    Air,
    Taxi,
    Other(u16),
}

impl Default for RouteType {
    fn default() -> RouteType {
        RouteType::Bus
    }
}

impl<'de> Deserialize<'de> for RouteType {
    fn deserialize<D>(deserializer: D) -> Result<RouteType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let i = u16::deserialize(deserializer)?;

        let hundreds = i / 100;
        Ok(match (i, hundreds) {
            (0, _) | (_, 9) => RouteType::Tramway,
            (1, _) | (_, 4) => RouteType::Subway,
            (2, _) | (_, 1) => RouteType::Rail,
            (3, _) | (_, 7) | (_, 8) => RouteType::Bus,
            (4, _) | (_, 10) | (_, 12) => RouteType::Ferry,
            (5, _) => RouteType::CableCar,
            (6, _) | (_, 13) => RouteType::Gondola,
            (7, _) | (_, 14) => RouteType::Funicular,
            (_, 2) => RouteType::Coach,
            (_, 11) => RouteType::Air,
            (_, 15) => RouteType::Taxi,
            _ => RouteType::Other(i),
        })
    }
}

impl Serialize for RouteType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Note: for extended route type, we might loose the initial precise route type
        serializer.serialize_u16(match self {
            RouteType::Tramway => 0,
            RouteType::Subway => 1,
            RouteType::Rail => 2,
            RouteType::Bus => 3,
            RouteType::Ferry => 4,
            RouteType::CableCar => 5,
            RouteType::Gondola => 6,
            RouteType::Funicular => 7,
            RouteType::Coach => 200,
            RouteType::Air => 1100,
            RouteType::Taxi => 1500,
            RouteType::Other(i) => *i,
        })
    }
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum PickupDropOffType {
    #[derivative(Default)]
    #[serde(rename = "0")]
    Regular,
    #[serde(rename = "1")]
    NotAvailable,
    #[serde(rename = "2")]
    ArrangeByPhone,
    #[serde(rename = "3")]
    CoordinateWithDriver,
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum ContinuousPickupDropOff {
    #[serde(rename = "0")]
    Continuous,
    #[derivative(Default)]
    #[serde(rename = "1")]
    NotAvailable,
    #[serde(rename = "2")]
    ArrangeByPhone,
    #[serde(rename = "3")]
    CoordinateWithDriver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Calendar {
    #[serde(rename = "service_id")]
    pub id: String,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub monday: bool,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub tuesday: bool,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub wednesday: bool,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub thursday: bool,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub friday: bool,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub saturday: bool,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub sunday: bool,
    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    pub start_date: NaiveDate,
    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    pub end_date: NaiveDate,
}

impl Type for Calendar {
    fn object_type(&self) -> ObjectType {
        ObjectType::Calendar
    }
}

impl Id for Calendar {
    fn id(&self) -> &str {
        &self.id
    }
}

impl fmt::Display for Calendar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}—{}", self.start_date, self.end_date)
    }
}

#[derive(Serialize, Deserialize, Debug, Derivative, PartialEq, Eq, Hash, Clone, Copy)]
#[derivative(Default)]
pub enum Availability {
    #[derivative(Default)]
    #[serde(rename = "0")]
    InformationNotAvailable,
    #[serde(rename = "1")]
    Available,
    #[serde(rename = "2")]
    NotAvailable,
}

impl Calendar {
    pub fn valid_weekday(&self, date: NaiveDate) -> bool {
        match date.weekday() {
            Weekday::Mon => self.monday,
            Weekday::Tue => self.tuesday,
            Weekday::Wed => self.wednesday,
            Weekday::Thu => self.thursday,
            Weekday::Fri => self.friday,
            Weekday::Sat => self.saturday,
            Weekday::Sun => self.sunday,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Exception {
    #[serde(rename = "1")]
    Added,
    #[serde(rename = "2")]
    Deleted,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CalendarDate {
    pub service_id: String,
    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    pub date: NaiveDate,
    pub exception_type: Exception,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Stop {
    #[serde(rename = "stop_id")]
    pub id: String,
    #[serde(rename = "stop_code")]
    pub code: Option<String>,
    #[serde(rename = "stop_name")]
    pub name: String,
    #[serde(default, rename = "stop_desc")]
    pub description: String,
    #[serde(default = "default_location_type")]
    pub location_type: LocationType,
    pub parent_station: Option<String>,
    pub zone_id: Option<String>,
    #[serde(rename = "stop_url")]
    pub url: Option<String>,
    #[serde(deserialize_with = "de_with_optional_float")]
    #[serde(rename = "stop_lon", default)]
    pub longitude: Option<f64>,
    #[serde(deserialize_with = "de_with_optional_float")]
    #[serde(rename = "stop_lat", default)]
    pub latitude: Option<f64>,
    #[serde(rename = "stop_timezone")]
    pub timezone: Option<String>,
    #[serde(deserialize_with = "de_with_empty_default", default)]
    pub wheelchair_boarding: Availability,
    pub level_id: Option<String>,
    pub platform_code: Option<String>,
}

impl Type for Stop {
    fn object_type(&self) -> ObjectType {
        ObjectType::Stop
    }
}

impl Id for Stop {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Translatable for Stop {
    fn translate(&self, gtfs: &Gtfs, language: &str) -> Self {
        Stop {
            id: self.id.clone(),
            code: self.code.as_ref().map(|code| 
                gtfs.translate(
                    "stops",
                    "stop_code",
                    language,
                    &self.id,
                    None,
                    &code
                )
            ),
            name: gtfs.translate(
                "stops",
                "stop_name",
                language,
                &self.id,
                None,
                &self.name
            ),
            description: gtfs.translate(
                "stops",
                "stop_desc",
                language,
                &self.id,
                None,
                &self.description
            ),
            location_type: self.location_type,
            parent_station: self.parent_station.clone(),
            zone_id: self.zone_id.clone(),
            url: self.code.as_ref().map(|url|
                gtfs.translate(
                    "stops",
                    "stop_url",
                    language,
                    &self.id,
                    None,
                    &url
                )
            ),
            longitude: self.longitude,
            latitude: self.latitude,
            timezone: self.timezone.clone(),
            wheelchair_boarding: self.wheelchair_boarding,
            level_id: self.level_id.clone(),
            platform_code: self.code.as_ref().map(|platform_code|
                gtfs.translate(
                    "stops",
                    "platform_code",
                    language,
                    &self.id,
                    None,
                    &platform_code
                )
            ),
        }
    }
}

impl fmt::Display for Stop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RawStopTime {
    pub trip_id: String,
    /// Arrival time of the stop time.
    /// It's an option since the intermediate stops can have have no arrival
    /// and this arrival needs to be interpolated
    #[serde(
        deserialize_with = "deserialize_optional_time",
        serialize_with = "serialize_optional_time"
    )]
    pub arrival_time: Option<u32>,
    /// Departure time of the stop time.
    /// It's an option since the intermediate stops can have have no departure
    /// and this departure needs to be interpolated
    #[serde(
        deserialize_with = "deserialize_optional_time",
        serialize_with = "serialize_optional_time"
    )]
    pub departure_time: Option<u32>,
    pub stop_id: String,
    pub stop_sequence: u16,
    pub stop_headsign: Option<String>,
    pub pickup_type: Option<PickupDropOffType>,
    pub drop_off_type: Option<PickupDropOffType>,
    pub continuous_pickup: Option<ContinuousPickupDropOff>,
    pub continuous_drop_off: Option<ContinuousPickupDropOff>,
    pub shape_dist_traveled: Option<f32>,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool",
        default = "bool_default_true"
    )]
    pub timepoint: bool,
}

#[derive(Debug, Default)]
pub struct StopTime {
    pub arrival_time: Option<u32>,
    pub stop: Arc<Stop>,
    pub departure_time: Option<u32>,
    pub pickup_type: Option<PickupDropOffType>,
    pub drop_off_type: Option<PickupDropOffType>,
    pub stop_sequence: u16,
    pub stop_headsign: Option<String>,
    pub continuous_pickup: Option<ContinuousPickupDropOff>,
    pub continuous_drop_off: Option<ContinuousPickupDropOff>,
    pub shape_dist_traveled: Option<f32>,
    pub timepoint: bool,
}

impl Translatable for StopTime {
    fn translate(&self, gtfs: &Gtfs, language: &str) -> Self {
        StopTime {
            arrival_time: self.arrival_time.clone(),
            stop: Arc::new(self.stop.translate(gtfs, language)),
            departure_time: self.departure_time.clone(),
            pickup_type: self.pickup_type.clone(),
            drop_off_type: self.drop_off_type.clone(),
            stop_sequence: self.stop_sequence,
            // Headsign can't be translated as we do not have a reference to this StopTime's Trip
            stop_headsign: self.stop_headsign.clone(),
            continuous_pickup: self.continuous_pickup.clone(),
            continuous_drop_off: self.continuous_drop_off.clone(),
            shape_dist_traveled: self.shape_dist_traveled,
            timepoint: self.timepoint
        }
    }
}

impl StopTime {
    pub fn from(stop_time_gtfs: &RawStopTime, stop: Arc<Stop>) -> Self {
        Self {
            arrival_time: stop_time_gtfs.arrival_time,
            departure_time: stop_time_gtfs.departure_time,
            stop,
            pickup_type: stop_time_gtfs.pickup_type,
            drop_off_type: stop_time_gtfs.drop_off_type,
            stop_sequence: stop_time_gtfs.stop_sequence,
            stop_headsign: stop_time_gtfs.stop_headsign.clone(),
            continuous_pickup: stop_time_gtfs.continuous_pickup,
            continuous_drop_off: stop_time_gtfs.continuous_drop_off,
            shape_dist_traveled: stop_time_gtfs.shape_dist_traveled,
            timepoint: stop_time_gtfs.timepoint,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Route {
    #[serde(rename = "route_id")]
    pub id: String,
    #[serde(rename = "route_short_name")]
    pub short_name: String,
    #[serde(rename = "route_long_name")]
    pub long_name: String,
    #[serde(rename = "route_desc")]
    pub desc: Option<String>,
    pub route_type: RouteType,
    #[serde(rename = "route_url")]
    pub url: Option<String>,
    pub agency_id: Option<String>,
    #[serde(rename = "route_sort_order")]
    pub route_order: Option<u32>,
    #[serde(
        deserialize_with = "de_with_optional_color",
        serialize_with = "serialize_optional_color",
        default
    )]
    pub route_color: Option<RGB8>,
    #[serde(
        deserialize_with = "de_with_optional_color",
        serialize_with = "serialize_optional_color",
        default
    )]
    pub route_text_color: Option<RGB8>,
    pub continuous_pickup: Option<ContinuousPickupDropOff>,
    pub continuous_drop_off: Option<ContinuousPickupDropOff>,
}

impl Type for Route {
    fn object_type(&self) -> ObjectType {
        ObjectType::Route
    }
}

impl Id for Route {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Translatable for Route {
    fn translate(&self, gtfs: &Gtfs, language: &str) -> Route {
        Route {
            id: self.id.clone(),
            short_name: gtfs.translate(
                "routes",
                "route_short_name",
                language,
                &self.id,
                None,
                &self.short_name
            ),
            long_name: gtfs.translate(
                "routes",
                "route_long_name",
                language,
                &self.id,
                None,
                &self.long_name
            ),
            desc: self.desc.as_ref().map(|desc| gtfs.translate(
                    "routes",
                    "route_desc",
                    language,
                    &self.id,
                    None,
                    &desc
            )),
            route_type: self.route_type.clone(),
            url: self.url.as_ref().map(|url| gtfs.translate(
                    "routes",
                    "route_url",
                    language,
                    &self.id,
                    None,
                    &url
            )),
            agency_id: self.agency_id.clone(),
            route_order: self.route_order.clone(),
            route_color: self.route_color.clone(),
            route_text_color: self.route_text_color.clone(),
            continuous_pickup: self.continuous_pickup.clone(),
            continuous_drop_off: self.continuous_drop_off.clone(),
        }
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.long_name.is_empty() {
            write!(f, "{}", self.long_name)
        } else {
            write!(f, "{}", self.short_name)
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq)]
pub enum DirectionType {
    #[serde(rename = "0")]
    Outbound,
    #[serde(rename = "1")]
    Inbound,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq)]
pub enum WheelChairAccessibleType {
    #[serde(rename = "0")]
    NoAccessibilityInfo,
    #[serde(rename = "1")]
    AtLeastOneWheelChair,
    #[serde(rename = "2")]
    NotWheelChairAccessible,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq)]
pub enum BikesAllowedType {
    #[serde(rename = "0")]
    NoBikeInfo,
    #[serde(rename = "1")]
    AtLeastOneBike,
    #[serde(rename = "2")]
    NoBikesAllowed,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RawTrip {
    #[serde(rename = "trip_id")]
    pub id: String,
    pub service_id: String,
    pub route_id: String,
    pub shape_id: Option<String>,
    pub trip_headsign: Option<String>,
    pub trip_short_name: Option<String>,
    pub direction_id: Option<DirectionType>,
    pub block_id: Option<String>,
    pub wheelchair_accessible: Option<WheelChairAccessibleType>,
    pub bikes_allowed: Option<BikesAllowedType>,
}

impl Type for RawTrip {
    fn object_type(&self) -> ObjectType {
        ObjectType::Trip
    }
}

impl Id for RawTrip {
    fn id(&self) -> &str {
        &self.id
    }
}

impl fmt::Display for RawTrip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "route id: {}, service id: {}",
            self.route_id, self.service_id
        )
    }
}

#[derive(Debug, Default)]
pub struct Trip {
    pub id: String,
    pub service_id: String,
    pub route_id: String,
    pub stop_times: Vec<StopTime>,
    pub shape_id: Option<String>,
    pub trip_headsign: Option<String>,
    pub trip_short_name: Option<String>,
    pub direction_id: Option<DirectionType>,
    pub block_id: Option<String>,
    pub wheelchair_accessible: Option<WheelChairAccessibleType>,
    pub bikes_allowed: Option<BikesAllowedType>,
}

impl Type for Trip {
    fn object_type(&self) -> ObjectType {
        ObjectType::Trip
    }
}

impl Id for Trip {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Translatable for Trip {
    fn translate(&self, gtfs: &Gtfs, language: &str) -> Self {
        Trip {
            id: self.id.clone(),
            service_id: self.service_id.clone(),
            route_id: self.route_id.clone(),
            stop_times: self.stop_times.iter().map(|stop_time| stop_time.translate(gtfs, language)).collect(),
            shape_id: self.shape_id.clone(),
            trip_headsign: self.trip_headsign.as_ref().map(|headsign| gtfs.translate(
                "trips",
                "trip_headsign",
                language,
                &self.id,
                None,
                &headsign
            )),
            trip_short_name: self.trip_short_name.as_ref().map(|short_name| gtfs.translate(
                "trips",
                "trip_short_name",
                language,
                &self.id,
                None,
                &short_name
            )),
            direction_id: self.direction_id.clone(),
            block_id: self.block_id.clone(),
            wheelchair_accessible: self.wheelchair_accessible.clone(),
            bikes_allowed: self.bikes_allowed.clone(),
        }
    }
}

impl fmt::Display for Trip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "route id: {}, service id: {}",
            self.route_id, self.service_id
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Agency {
    #[serde(rename = "agency_id")]
    pub id: Option<String>,
    #[serde(rename = "agency_name")]
    pub name: String,
    #[serde(rename = "agency_url")]
    pub url: String,
    #[serde(rename = "agency_timezone")]
    pub timezone: String,
    #[serde(rename = "agency_lang")]
    pub lang: Option<String>,
    #[serde(rename = "agency_phone")]
    pub phone: Option<String>,
    #[serde(rename = "agency_fare_url")]
    pub fare_url: Option<String>,
    #[serde(rename = "agency_email")]
    pub email: Option<String>,
}

impl Type for Agency {
    fn object_type(&self) -> ObjectType {
        ObjectType::Agency
    }
}

impl Id for Agency {
    fn id(&self) -> &str {
        match &self.id {
            None => "",
            Some(id) => id,
        }
    }
}

impl fmt::Display for Agency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Shape {
    #[serde(rename = "shape_id")]
    pub id: String,
    #[serde(rename = "shape_pt_lat", default)]
    pub latitude: f64,
    #[serde(rename = "shape_pt_lon", default)]
    pub longitude: f64,
    #[serde(rename = "shape_pt_sequence")]
    pub sequence: usize,
    #[serde(rename = "shape_dist_traveled")]
    pub dist_traveled: Option<f32>,
}

impl Type for Shape {
    fn object_type(&self) -> ObjectType {
        ObjectType::Shape
    }
}

impl Id for Shape {
    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FareAttribute {
    #[serde(rename = "fare_id")]
    pub id: String,
    pub price: String,
    #[serde(rename = "currency_type")]
    pub currency: String,
    pub payment_method: PaymentMethod,
    pub transfers: Transfers,
    pub agency_id: Option<String>,
    pub transfer_duration: Option<usize>,
}

impl Id for FareAttribute {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Type for FareAttribute {
    fn object_type(&self) -> ObjectType {
        ObjectType::Fare
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq)]
pub enum PaymentMethod {
    #[serde(rename = "0")]
    Aboard,
    #[serde(rename = "1")]
    PreBoarding,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Transfers {
    Unlimited,
    NoTransfer,
    UniqueTransfer,
    TwoTransfers,
    Other(u16),
}

impl<'de> Deserialize<'de> for Transfers {
    fn deserialize<D>(deserializer: D) -> Result<Transfers, D::Error>
    where
        D: Deserializer<'de>,
    {
        let i = Option::<u16>::deserialize(deserializer)?;
        Ok(match i {
            Some(0) => Transfers::NoTransfer,
            Some(1) => Transfers::UniqueTransfer,
            Some(2) => Transfers::TwoTransfers,
            Some(a) => Transfers::Other(a),
            None => Transfers::default(),
        })
    }
}

impl Serialize for Transfers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Transfers::NoTransfer => serializer.serialize_u16(0),
            Transfers::UniqueTransfer => serializer.serialize_u16(1),
            Transfers::TwoTransfers => serializer.serialize_u16(2),
            Transfers::Other(a) => serializer.serialize_u16(*a),
            Transfers::Unlimited => serializer.serialize_none(),
        }
    }
}

impl Default for Transfers {
    fn default() -> Transfers {
        Transfers::Unlimited
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedInfo {
    #[serde(rename = "feed_publisher_name")]
    pub name: String,
    #[serde(rename = "feed_publisher_url")]
    pub url: String,
    #[serde(rename = "feed_lang")]
    pub lang: String,
    pub default_lang: Option<String>,
    #[serde(
        deserialize_with = "deserialize_option_date",
        serialize_with = "serialize_option_date",
        rename = "feed_start_date",
        default
    )]
    pub start_date: Option<NaiveDate>,
    #[serde(
        deserialize_with = "deserialize_option_date",
        serialize_with = "serialize_option_date",
        rename = "feed_end_date",
        default
    )]
    pub end_date: Option<NaiveDate>,
    #[serde(rename = "feed_version")]
    pub version: Option<String>,
    #[serde(rename = "feed_contact_email")]
    pub contact_email: Option<String>,
    #[serde(rename = "feed_contact_url")]
    pub contact_url: Option<String>,
}

impl fmt::Display for FeedInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y%m%d").map_err(serde::de::Error::custom)
}

fn serialize_date<'ser, S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(format!("{}{}{}", date.year(), date.month(), date.day()).as_str())
}

fn deserialize_option_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?
        .map(|s| NaiveDate::parse_from_str(&s, "%Y%m%d").map_err(serde::de::Error::custom));
    match s {
        Some(Ok(s)) => Ok(Some(s)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

fn serialize_option_date<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        None => serializer.serialize_none(),
        Some(d) => {
            serializer.serialize_str(format!("{}{}{}", d.year(), d.month(), d.day()).as_str())
        }
    }
}

fn parse_time_impl(v: Vec<&str>) -> Result<u32, std::num::ParseIntError> {
    Ok(&v[0].parse()? * 3600u32 + &v[1].parse()? * 60u32 + &v[2].parse()?)
}

pub fn parse_time(s: &str) -> Result<u32, crate::Error> {
    let v: Vec<&str> = s.trim_start().split(':').collect();
    if v.len() != 3 {
        Err(crate::Error::InvalidTime(s.to_owned()))
    } else {
        Ok(parse_time_impl(v).map_err(|_| crate::Error::InvalidTime(s.to_owned()))?)
    }
}

fn deserialize_optional_time<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;

    match s {
        None => Ok(None),
        Some(t) => Ok(Some(parse_time(&t).map_err(de::Error::custom)?)),
    }
}

fn serialize_optional_time<S>(time: &Option<u32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match time {
        None => serializer.serialize_none(),
        Some(t) => serializer.serialize_str(format!("{}", t).as_str()),
    }
}

fn de_with_optional_float<'de, D>(de: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(de).and_then(|s| {
        let s = s.trim();
        if s == "" {
            Ok(None)
        } else {
            s.parse().map(Some).map_err(de::Error::custom)
        }
    })
}

pub fn parse_color(s: &str) -> Result<RGB8, crate::Error> {
    if s.len() != 6 {
        return Err(crate::Error::InvalidColor(s.to_owned()));
    }
    let r =
        u8::from_str_radix(&s[0..2], 16).map_err(|_| crate::Error::InvalidColor(s.to_owned()))?;
    let g =
        u8::from_str_radix(&s[2..4], 16).map_err(|_| crate::Error::InvalidColor(s.to_owned()))?;
    let b =
        u8::from_str_radix(&s[4..6], 16).map_err(|_| crate::Error::InvalidColor(s.to_owned()))?;
    Ok(RGB8::new(r, g, b))
}

fn de_with_optional_color<'de, D>(de: D) -> Result<Option<RGB8>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(de).and_then(|s| {
        let s = s.trim();
        if s == "" {
            Ok(None)
        } else {
            parse_color(s).map(Some).map_err(de::Error::custom)
        }
    })
}

fn serialize_optional_color<S>(color: &Option<RGB8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match color {
        None => serializer.serialize_none(),
        Some(RGB8 { r, g, b }) => {
            serializer.serialize_str(format!("{:02X}{:02X}{:02X}", r, g, b).as_str())
        }
    }
}

pub fn de_with_empty_default<'de, T: Default, D>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(de).map(|opt| opt.unwrap_or_else(Default::default))
}

fn default_location_type() -> LocationType {
    LocationType::StopPoint
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match &*s {
        "0" => Ok(false),
        "1" => Ok(true),
        &_ => Err(serde::de::Error::custom(format!(
            "Invalid value `{}`, expected 0 or 1",
            s
        ))),
    }
}

fn bool_default_true() -> bool {
    true
}

fn serialize_bool<'ser, S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *value {
        serializer.serialize_u8(1)
    } else {
        serializer.serialize_u8(0)
    }
}
