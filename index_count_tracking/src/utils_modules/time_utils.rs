use crate::common::*;

#[doc = ""]
pub fn calc_struct_to_strkor(utc_time: &str) -> anyhow::Result<String> {
    let dt_utc: DateTime<Utc> = utc_time.parse::<DateTime<Utc>>()?;
    let dt_local: DateTime<Local> = dt_utc.with_timezone(&Local);
    Ok(dt_local.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

#[doc = "duration 이전 시각을 반환해주는 함수"]
pub fn calc_time_window(dt: DateTime<Utc>, duration_secs: i64) -> DateTime<Utc> {
    dt - chrono::Duration::seconds(duration_secs)
}

#[doc = ""]
pub fn convert_utc_from_local(local_time: DateTime<Local>) -> DateTime<Utc> {
    local_time.with_timezone(&Utc)
}

#[doc = ""]
pub fn convert_local_from_utc(utc_time: DateTime<Utc>) -> DateTime<Local> {
    utc_time.with_timezone(&Local)
}

#[doc = ""]
pub fn convert_date_to_str<Tz, TzOut>(
    time: DateTime<Tz>,
    tz: TzOut, // 출력할 타임존 (Utc, Local, FixedOffset 등)
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

#[doc = "특정 시각에서 특정 시각을 빼준 시각을 반환하는 함수"]
pub fn minus_h(dt: DateTime<Utc>, hours: i64) -> DateTime<Utc> {
    dt - chrono::Duration::hours(hours)
}
