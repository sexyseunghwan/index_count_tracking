use crate::common::*;

#[doc = r#"
    환경변수를 읽어와서 반환하고, 환경변수가 설정되지 않은 경우 치명적 오류로 처리하는 함수.

    애플리케이션의 필수 설정값들이 환경변수로 관리되므로, 해당 환경변수가 없으면
    애플리케이션이 정상 동작할 수 없기 때문에 panic으로 즉시 종료시킨다.

    1. 환경변수 `key`에 해당하는 값을 `env::var()`로 조회
    2. 값이 존재하면 해당 값을 문자열로 반환
    3. 값이 없으면:
       - 에러 메시지를 구성하여 error 레벨로 로깅
       - 동일한 메시지로 panic 발생시켜 애플리케이션 종료

    # Arguments
    * `key` - 조회할 환경변수 키명

    # Returns
    * `String` - 환경변수 값

    # Panics
    환경변수가 설정되지 않은 경우 애플리케이션 종료
"#]
fn get_env_or_panic(key: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => {
            let msg = format!("[ENV file read Error] '{}' must be set", key);
            error!("{}", msg);
            panic!("{}", msg);
        }
    }
}

#[doc = r#"
    모니터링 대상 인덱스 목록 설정 파일의 경로를 환경변수에서 읽어와 전역 변수로 초기화.

    `INDEX_LIST_PATH` 환경변수를 통해 TOML 형식의 인덱스 설정 파일 경로를 지정받는다.
    이 파일에는 각 인덱스의 모니터링 설정(허용 변동률, 집계 주기 등)이 포함되어 있다.
    once_lazy를 사용하여 첫 접근 시에만 초기화되며, 이후에는 캐시된 값을 재사용한다.

    # 예상 파일 내용
    인덱스별 모니터링 설정 정보 (TOML 형식)

    # Panics
    `INDEX_LIST_PATH` 환경변수가 설정되지 않은 경우
"#]
pub static INDEX_LIST_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("INDEX_LIST_PATH"));

#[doc = r#"
    알람 이메일 수신자 목록 설정 파일의 경로를 환경변수에서 읽어와 전역 변수로 초기화.

    `EMAIL_RECEIVER_PATH` 환경변수를 통해 TOML 형식의 수신자 목록 파일 경로를 지정받는다.
    이 파일에는 인덱스 알람을 받을 이메일 주소들이 포함되어 있다.
    once_lazy를 사용하여 첫 접근 시에만 초기화되며, 이후에는 캐시된 값을 재사용한다.

    # 예상 파일 내용
    이메일 수신자 목록 설정 (TOML 형식)

    # Panics
    `EMAIL_RECEIVER_PATH` 환경변수가 설정되지 않은 경우
"#]
pub static EMAIL_RECEIVER_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("EMAIL_RECEIVER_PATH"));

#[doc = r#"
    서버 설정 정보 파일의 경로를 환경변수에서 읽어와 전역 변수로 초기화.

    `SERVER_CONFIG_PATH` 환경변수를 통해 TOML 형식의 서버 설정 파일 경로를 지정받는다.
    이 파일에는 Elasticsearch 연결 정보, Telegram 설정, SQL Server 설정, 시스템 설정 등
    애플리케이션 실행에 필요한 모든 서버 설정 정보가 포함되어 있다.
    once_lazy를 사용하여 첫 접근 시에만 초기화되며, 이후에는 캐시된 값을 재사용한다.

    # 예상 파일 내용
    - Elasticsearch 클러스터 정보
    - 모니터링 Elasticsearch 정보
    - Telegram 봇 설정
    - SQL Server 연결 정보
    - 시스템 설정 (메시지 청크 크기 등)

    # Panics
    `SERVER_CONFIG_PATH` 환경변수가 설정되지 않은 경우
"#]
pub static SERVER_CONFIG_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("SERVER_CONFIG_PATH"));

#[doc = r#"
    이메일 알람용 HTML 템플릿 파일의 경로를 환경변수에서 읽어와 전역 변수로 초기화.

    `HTML_TEMPLATE_PATH` 환경변수를 통해 HTML 템플릿 파일 경로를 지정받는다.
    이 템플릿 파일은 인덱스 알람 이메일의 레이아웃과 스타일을 정의하며,
    플레이스홀더를 통해 동적으로 알람 데이터가 삽입된다.
    once_lazy를 사용하여 첫 접근 시에만 초기화되며, 이후에는 캐시된 값을 재사용한다.

    # 예상 템플릿 플레이스홀더
    - `{cluster_name}`: Elasticsearch 클러스터명
    - `{alert_time}`: 알람 발생 시간
    - `{alert_rows}`: 알람 데이터 테이블 행들

    # Panics
    `HTML_TEMPLATE_PATH` 환경변수가 설정되지 않은 경우
"#]
pub static HTML_TEMPLATE_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("HTML_TEMPLATE_PATH"));

// #[doc = "Function to globally initialize the 'INDEX_ALERT_TEMPLATE_PATH' variable"]
// pub static INDEX_ALERT_TEMPLATE_PATH: once_lazy<String> =
//     once_lazy::new(|| String::from("html/index_alert_template.html"));
