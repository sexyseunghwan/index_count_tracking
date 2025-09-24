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
pub fn calc_time_window(time: &str, duration_secs: i64) -> anyhow::Result<String> {
    /* 입력 문자열을 NaiveDateTime으로 파싱 */
    let base_time = NaiveDateTime::parse_from_str(time, "%Y-%m-%dT%H:%M:%SZ")
        .map_err(|e| anyhow!("[time_utils->calc_time_window] {}", e))?;

    /* duration_secs 만큼 빼기 */
    let new_time: NaiveDateTime = base_time - chrono::Duration::seconds(duration_secs);

    /* 다시 문자열로 변환 */
    Ok(new_time.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}