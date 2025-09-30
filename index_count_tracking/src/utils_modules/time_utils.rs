use crate::common::*;

// #[doc = "Functions that return the current UTC time -> NaiveDate"]
// pub fn get_current_utc_naivedate() -> NaiveDate {
//     let utc_now: DateTime<Utc> = Utc::now();
//     utc_now.date_naive()
// }

#[doc = "Functions that return the current UTC time -> NaiveDatetime"]
pub fn get_currnet_utc_naivedatetime() -> NaiveDateTime {
    let utc_now: DateTime<Utc> = Utc::now();
    utc_now.naive_local()
}

#[doc = "Function that returns the current UTC time as a string"]
pub fn get_current_utc_naivedatetime_str() -> String {
    let curr_time: NaiveDateTime = get_currnet_utc_naivedatetime();
    curr_time.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[doc = "duration 이전 시각을 반환해주는 함수"]
// pub fn calc_time_window(time: &str, duration_secs: i64) -> anyhow::Result<String> {
//     /* 입력 문자열을 NaiveDateTime으로 파싱 */
//     let base_time: NaiveDateTime = NaiveDateTime::parse_from_str(time, "%Y-%m-%dT%H:%M:%SZ")
//         .map_err(|e| anyhow!("[time_utils->calc_time_window] {}", e))?;

//     /* duration_secs 만큼 빼기 */
//     let new_time: NaiveDateTime = base_time - chrono::Duration::seconds(duration_secs);

//     /* 다시 문자열로 변환 */
//     Ok(new_time.format("%Y-%m-%dT%H:%M:%SZ").to_string())
// }

#[doc = "duration 이전 시각을 반환해주는 함수"]
pub fn calc_time_window(dt: DateTime<Utc>, duration_secs: i64) -> DateTime<Utc> {
    dt - chrono::Duration::seconds(duration_secs)
}

#[doc = "로컬 현재 시간을 UTC로 변환해 `YYYY-MM-DDTHH:MM:SSZ` 문자열로 반환"]
// pub fn local_now_to_utc_iso8601(now_local: DateTime<Local>) -> String {
//     let now_utc: DateTime<Utc> = now_local.with_timezone(&Utc);
//     now_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string()
// }
#[doc = ""]
pub fn convert_utc_from_local(now_local: DateTime<Local>) -> DateTime<Utc> {
    now_local.with_timezone(&Utc)
}

#[doc = ""]
pub fn convert_date_to_str(utc_time: DateTime<Utc>) -> String {
    utc_time.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[doc = "특정 시각에서 특정 시각을 빼준 시각을 반환하는 함수"]
pub fn minus_h(dt: DateTime<Utc>, hours: i64) -> DateTime<Utc> {
    dt - chrono::Duration::hours(hours)
}
