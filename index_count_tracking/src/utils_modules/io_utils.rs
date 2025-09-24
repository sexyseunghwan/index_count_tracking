use crate::common::*;

#[doc = r#"
    TOML 형식의 설정 파일을 읽어와서 지정된 구조체 타입으로 역직렬화하는 제네릭 함수.

    애플리케이션의 다양한 설정 파일들(인덱스 설정, 이메일 수신자, 서버 설정 등)을
    TOML 형식으로 관리하며, 이 함수를 통해 타입 안전하게 구조체로 변환한다.

    1. 지정된 경로의 TOML 파일을 문자열로 읽어온다
    2. `toml::from_str()`을 사용하여 TOML 문자열을 제네릭 타입 T로 파싱
    3. serde의 역직렬화 기능을 활용하여 구조체로 변환
    4. 파일 읽기나 파싱 실패 시 적절한 오류 반환

    # Type Parameters
    * `T` - `DeserializeOwned` 트레이트를 구현한 구조체 타입

    # Arguments
    * `file_path` - 읽을 TOML 파일의 절대 경로 또는 상대 경로

    # Returns
    * `Result<T, anyhow::Error>` - 성공 시 파싱된 구조체, 실패 시 오류

    # Errors
    - 파일이 존재하지 않거나 읽기 권한이 없는 경우
    - TOML 형식이 잘못되어 파싱에 실패하는 경우
    - 구조체 필드와 TOML 키가 일치하지 않는 경우

    # Examples
    ```rust
    let config: ServerConfig = read_toml_from_file("config/server.toml")?;
    let index_list: IndexListConfig = read_toml_from_file(&INDEX_LIST_PATH)?;
    ```
"#]
/// # Arguments
/// * `file_path` - 읽을 대상 toml 파일이 존재하는 경로
///
/// # Returns
/// * Result<T, anyhow::Error> - 성공적으로 파일을 읽었을 경우에는 json 호환 객체를 반환해준다.
pub fn read_toml_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, anyhow::Error> {
    let toml_content = std::fs::read_to_string(file_path)?;
    let toml: T = toml::from_str(&toml_content)?;

    Ok(toml)
}

#[doc = r#"
    구조체를 JSON Value 객체로 변환하는 제네릭 유틸리티 함수.

    Elasticsearch 쿼리나 다른 JSON API 호출에서 구조체 데이터를
    serde_json::Value 형태로 변환할 때 사용한다.

    1. serde의 직렬화 기능을 사용하여 구조체를 JSON Value로 변환
    2. `serde_json::to_value()`를 통해 안전하게 변환 수행
    3. 변환 실패 시 상세한 오류 메시지와 함께 anyhow::Error 반환
    4. 성공 시 serde_json::Value 객체 반환

    이 함수는 주로 다음과 같은 상황에서 사용됨:
    - Elasticsearch 쿼리 구성 시 구조체를 JSON으로 변환
    - API 요청 페이로드 생성
    - 로깅이나 디버깅 목적의 JSON 출력

    # Type Parameters
    * `T` - `Serialize` 트레이트를 구현한 구조체 타입

    # Arguments
    * `input_struct` - JSON으로 변환할 구조체의 참조

    # Returns
    * `Result<Value, anyhow::Error>` - 성공 시 JSON Value, 실패 시 오류

    # Errors
    구조체의 필드가 JSON으로 직렬화될 수 없는 타입을 포함하는 경우

    # Examples
    ```rust
    let alert_index = AlertIndex::new("test_index".to_string(), 100, "2023-01-01".to_string());
    let json_value = convert_json_from_struct(&alert_index)?;
    ```
"#]
/// # Arguments
/// * input_struct - json 으로 변환할 구조체
///
/// # Returns
/// * Result<Value, anyhow::Error>
pub fn convert_json_from_struct<T: Serialize>(input_struct: &T) -> Result<Value, anyhow::Error> {
    serde_json::to_value(input_struct).map_err(|err| {
        anyhow!(
            "[Error][convert_json_from_struct()] Failed to serialize struct to JSON: {}",
            err
        )
    })
}
