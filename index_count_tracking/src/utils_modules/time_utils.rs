use crate::common::*;

#[doc = "Functions that return the current UTC time -> NaiveDate"]
pub fn get_current_utc_naivedate() -> NaiveDate {
    let utc_now: DateTime<Utc> = Utc::now();
    utc_now.date_naive()
}

#[doc = "Functions that return the current UTC time -> NaiveDatetime"]
pub fn get_currnet_utc_naivedatetime() -> NaiveDateTime {
    let utc_now: DateTime<Utc> = Utc::now();
    utc_now.naive_local()
}

#[doc = "Functions that make the current date (Korean time) a 'NaiveDateTime' data type"]
pub fn get_current_kor_naive_datetime() -> NaiveDateTime {
    let utc_now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = utc_now.with_timezone(&Seoul);

    kst_time.naive_local()
}

#[doc = "현재 한국시간을 문자열로 반환해주는 함수"]
pub fn get_current_kor_naive_datetime_str() -> Result<String, anyhow::Error> {
    let cur_time: NaiveDateTime = get_current_kor_naive_datetime();
    let cur_time_str: String = get_str_from_naivedatetime(cur_time, "%Y-%m-%dT%H:%M:%SZ")?;
    Ok(cur_time_str)
}

#[doc = "Function that returns the current UTC time as a string"]
pub fn get_current_utc_naivedate_str(fmt: &str) -> Result<String, anyhow::Error> {
    let curr_time = get_current_utc_naivedate();
    get_str_from_naivedate(curr_time, fmt)
}

#[doc = "Function that converts the date data 'naivedate' format to the string format"]
pub fn get_str_from_naivedatetime(
    naive_date: NaiveDateTime,
    fmt: &str,
) -> Result<String, anyhow::Error> {
    let result_date = naive_date.format(fmt).to_string();
    Ok(result_date)
}

#[doc = "Function that converts the date data 'naivedate' format to the string format"]
pub fn get_str_from_naivedate(naive_date: NaiveDate, fmt: &str) -> Result<String, anyhow::Error> {
    let result_date: String = naive_date.format(fmt).to_string();
    Ok(result_date)
}

#[doc = "Function that converts the date data 'naivedatetime' format to String format"]
pub fn get_str_from_naive_datetime(
    naive_datetime: NaiveDateTime,
    fmt: &str,
) -> Result<String, anyhow::Error> {
    let result_date: String = naive_datetime.format(fmt).to_string();
    Ok(result_date)
}

#[doc = "현재 UTC와 duration 이전 시각 반환해주는 함수"]
pub fn calc_time_window(duration_secs: i64) -> (NaiveDateTime, NaiveDateTime) {
    let curr_time_utc: NaiveDateTime = get_currnet_utc_naivedatetime();
    let time_minutes_ago: NaiveDateTime = curr_time_utc - chrono::Duration::seconds(duration_secs);
    (curr_time_utc, time_minutes_ago)
}