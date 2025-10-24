use crate::common::*;

pub fn calc_struct_to_strkor(utc_time: &str) -> anyhow::Result<String> {
    let dt_utc: DateTime<Utc> = utc_time.parse::<DateTime<Utc>>()?;
    let dt_local: DateTime<Local> = dt_utc.with_timezone(&Local);
    Ok(dt_local.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

#[doc = "duration 이전 시각을 반환해주는 함수"]
pub fn calc_time_window(dt: DateTime<Utc>, duration_secs: i64) -> DateTime<Utc> {
    dt - chrono::Duration::seconds(duration_secs)
}

#[doc = "Function to convert string timestamp to UTC data."]
pub fn convert_utc_from_str(time_str: &str) -> anyhow::Result<DateTime<Utc>> {
    let utc_time: DateTime<Utc> = match time_str.parse::<DateTime<Utc>>() {
        Ok(utc_time) => utc_time,
        Err(e) => return Err(anyhow!("[convert_utc_from_str] {:?}", e)),
    };

    Ok(utc_time)
}

pub fn convert_utc_from_local(local_time: DateTime<Local>) -> DateTime<Utc> {
    local_time.with_timezone(&Utc)
}

pub fn convert_local_from_utc(utc_time: DateTime<Utc>) -> DateTime<Local> {
    utc_time.with_timezone(&Local)
}

pub fn convert_date_to_str<Tz, TzOut>(
    time: DateTime<Tz>,
    tz: TzOut, // Timezone (Utc, Local, FixedOffset ...)
) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
    TzOut: TimeZone,
    TzOut::Offset: Display,
{
    time.with_timezone(&tz)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

pub fn convert_data_to_str_human<Tz, TzOut>(time: DateTime<Tz>, tz: TzOut) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
    TzOut: TimeZone,
    TzOut::Offset: Display,
{
    time.with_timezone(&tz)
        .format("%Y.%m.%d %H:%M:%S")
        .to_string()
}

#[doc = "특정 시각에서 특정 시각을 빼준 시각을 반환하는 함수"]
pub fn minus_h(dt: DateTime<Utc>, hours: i64) -> DateTime<Utc> {
    dt - chrono::Duration::hours(hours)
}

#[doc = "특정 시각에서 특정 시각을 빼준 시각을 반환하는 함수"]
pub fn minus_h_local(dt: DateTime<Local>, hours: i64) -> DateTime<Local> {
    dt - chrono::Duration::hours(hours)
}
