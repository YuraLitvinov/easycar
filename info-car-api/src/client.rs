use std::{collections::HashMap, num::NonZeroU32};

use crate::error::*;
use crate::types::*;

use chrono::{DateTime, Duration, Utc};
use reqwest::ClientBuilder;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub sub: String,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
}

pub struct Client {
    client: reqwest::Client,
    token: Option<String>,
    pub token_expire_date: Option<DateTime<Utc>>,
}

impl Client {
    pub fn new() -> Self {
        Client {
            client: ClientBuilder::new()
                .use_rustls_tls()
                .cookie_store(true)
                .build()
                .unwrap(),
            token: None,
            token_expire_date: None,
        }
    }
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    fn get_token(&self) -> Result<&String, NoBearerError> {
        self.token.as_ref().ok_or(NoBearerError)
    }

    pub async fn refresh_token(&mut self) -> Result<(), RefreshTokenError> {
        let response = self
            .client
            .get("https://info-car.pl/oauth2/authorize")
            .query(&[
                ("response_type", "id_token token"),
                ("client_id", "client"),
                (
                    "redirect_uri",
                    "https://info-car.pl/new/assets/refresh.html",
                ),
                ("scope", "openid profile email resource.read"),
                ("prompt", "none"),
            ])
            .send()
            .await?;

        log::debug!("{response:?}");

        let parsed_response: HashMap<String, String> = serde_urlencoded::from_str(
            response
                .url()
                .fragment()
                .ok_or(RefreshTokenError::NoFragmentProvided)?,
        )?;

        let Some(new_token) = parsed_response.get("access_token") else {
            return Err(RefreshTokenError::AccessTokenNotProvided(parsed_response));
        };

        let expire_time_unix: i64 = parsed_response
            .get("expires_in")
            .ok_or(RefreshTokenError::ExpireTimeNotProvided)?
            .parse()
            .or(Err(RefreshTokenError::ExpireTimeParseError))?;

        self.token_expire_date = Some(Utc::now() + Duration::seconds(expire_time_unix));

        self.set_token(new_token.to_owned());

        Ok(())
    }

    async fn get_csrf_token(&self, url: &str) -> Result<String, CsrfTokenError> {
        let response = self.client.get(url).send().await?;

        log::debug!("{response:?}");

        let fragment = Html::parse_fragment(&response.text().await?);
        let csrf_selector =
            Selector::parse("input[type=\"hidden\"][name=\"_csrf\"]").expect("Wrong selector");

        let csrf_element = fragment
            .select(&csrf_selector)
            .next()
            .ok_or(CsrfTokenError::TokenNotFound)?;
        Ok(csrf_element
            .value()
            .attr("value")
            .ok_or(CsrfTokenError::TokenValueNotFound)?
            .to_owned())
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), LoginError> {
        let csrf_token = self
            .get_csrf_token("https://info-car.pl/oauth2/login")
            .await?;

        let form_params = [
            ("username", username),
            ("_csrf", &csrf_token),
            ("password", password),
            ("_csrf", &csrf_token),
        ];

        let login_response = self
            .client
            .post("https://info-car.pl/oauth2/login")
            .form(&form_params)
            .send()
            .await?;

        dbg!("Login response: {:?}", login_response);

        self.refresh_token().await?;

        Ok(())
    }

    pub async fn logout(&mut self) -> Result<(), LogoutError> {
        self.client
            .get(format!(
                "https://info-car.pl/oauth2/endsession?id_token_hint={}",
                self.get_token()?
            ))
            .send()
            .await?;
        self.token = None;
        Ok(())
    }

    pub async fn user_info(&self) -> Result<UserInfo, GenericClientError> {
        Ok(self
            .client
            .get("https://info-car.pl/oauth2/userinfo")
            .bearer_auth(self.get_token()?)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn my_reservations(&self) -> Result<ReservationList, GenericClientError> {
        let response = self
            .client
            .get("https://info-car.pl/api/word/reservations")
            .bearer_auth(self.get_token()?)
            .send()
            .await?;

        log::debug!("{response:?}");

        Ok(handle_response(response)?.json().await?)
    }

    pub async fn word_centers(&self) -> Result<WordCenters, GenericClientError> {
        let response = self
            .client
            .get("https://info-car.pl/api/word/word-centers")
            .bearer_auth(self.get_token()?)
            .send()
            .await?;

        log::debug!("{response:?}");

        Ok(handle_response(response)?.json().await?)
    }

    pub async fn is_word_reschedule_enabled(
        &self,
        word_id: NonZeroU32,
    ) -> Result<bool, EnrollError> {
        let response = self
            .client
            .get(format!(
                "https://info-car.pl/api/word/word-centers/reschedule-enabled/{word_id}"
            ))
            .bearer_auth(self.get_token()?)
            .send()
            .await?;

        log::debug!("{response:?}");

        Ok(handle_response(response)?
            .json::<EndpointResponse<WordRescheduleEnabled>>()
            .await?
            .ok()?
            .reschedule_enabled)
    }

    pub async fn exam_schedule(
        &self,
        word_id: NonZeroU32,
        end_date: DateTime<Utc>,
        start_date: DateTime<Utc>,
        category: LicenseCategory,
    ) -> Result<ExamSchedule, EnrollError> {
        let mut map = HashMap::<&str, String>::new();
        map.insert("category", category.to_string());
        map.insert("endDate", end_date.to_string());
        map.insert("startDate", start_date.to_string());
        map.insert("wordId", word_id.to_string());

        let response = self
            .client
            .put("https://info-car.pl/api/word/word-centers/exam-schedule")
            .bearer_auth(self.get_token()?)
            .json(&map)
            .send()
            .await?;

        log::debug!("{response:?}");

        Ok(handle_response(response)?
            .json::<EndpointResponse<ExamSchedule>>()
            .await?
            .ok()?)
    }

    pub async fn new_reservation(
        &self,
        reservation: NewReservation,
    ) -> Result<String, EnrollError> {
        let response = self
            .client
            .post("https://info-car.pl/api/word/reservations")
            .bearer_auth(self.get_token()?)
            .json(&reservation)
            .send()
            .await?;

        log::debug!("{response:?}");

        Ok(handle_response(response)?
            .json::<EndpointResponse<NewReservationSuccess>>()
            .await?
            .ok()?
            .id)
    }

    pub async fn reservation_status(
        &self,
        reservation_id: String,
    ) -> Result<ReservationStatus, EnrollError> {
        if reservation_id.is_empty() {
            return Err(EnrollError::EmptyArg("reservation_id".to_string()));
        }
        let response = self
            .client
            .get(format!(
                "https://info-car.pl/api/word/reservations/{reservation_id}"
            ))
            .bearer_auth(self.get_token()?)
            .send()
            .await?;

        log::debug!("{response:?}");

        Ok(handle_response(response)?
            .json::<EndpointResponse<ReservationStatus>>()
            .await?
            .ok()?)
    }

    pub async fn cancel_reservation(&self, reservation_id: String) -> Result<(), EnrollError> {
        if reservation_id.is_empty() {
            return Err(EnrollError::EmptyArg("reservation_id".to_string()));
        }
        let response = self
            .client
            .post(format!(
                "https://info-car.pl/api/word/reservations/{reservation_id}/cancel"
            ))
            .bearer_auth(self.get_token()?)
            .send()
            .await?;

        log::debug!("{response:?}");

        Ok(handle_response(response)?
            .json::<EndpointResponse<()>>()
            .await?
            .ok()?)
    }

    pub async fn pay_with_blik(
        &self,
        reservation_id: String,
        blik_code: String,
        balance_usage: bool,
    ) -> Result<BlikPaymentResponse, EnrollError> {
        if reservation_id.is_empty() {
            return Err(EnrollError::EmptyArg("reservation_id".to_string()));
        }
        let request = BlikPaymentRequest::new(blik_code, balance_usage);
        let response = self
            .client
            .post(format!(
                "https://info-car.pl/api/word/reservations/{reservation_id}/blik"
            ))
            .json(&request)
            .bearer_auth(self.get_token()?)
            .send()
            .await?;

        log::debug!("{response:?}");

        Ok(handle_response(response)?
            .json::<EndpointResponse<BlikPaymentResponse>>()
            .await?
            .ok()?)
    }

    pub async fn list_applications(
        &self,
        employer_id: String,
    ) -> Result<Value, GenericClientError> {
        if employer_id.is_empty() {
            return Err(GenericClientError::ValidationError(
                "employer_id cannot be empty".to_string(),
            ));
        }

        let response = self
            .client
            .get(format!(
                "https://info-car.pl/api/stc/employers/{employer_id}/applications/"
            ))
            .bearer_auth(self.get_token()?)
            .send()
            .await?;

        log::debug!("{response:?}");

        Ok(handle_response(response)?.json().await?)
    }

    pub async fn add_employee(
        &self,
        employer_id: String,
        employee: AddEmployeeRequest,
    ) -> Result<Value, GenericClientError> {
        if employer_id.is_empty() {
            return Err(GenericClientError::ValidationError(
                "employer_id cannot be empty".to_string(),
            ));
        }

        let response = self
            .client
            .post(format!(
                "https://info-car.pl/api/stc/employers/{employer_id}/employees/"
            ))
            .header(
                "Content-Type",
                "application/vnd.pwpw.infocar.stc.public.v1+json",
            )
            .bearer_auth(self.get_token()?)
            .json(&employee)
            .send()
            .await?;

        log::debug!("{response:?}");

        Ok(handle_response(response)?.json().await?)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_applications() {
        dotenvy::dotenv().ok();
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        println!("Current directory: {}", current_dir.display());

        let username = dotenvy::var("USER_HANDLER").expect("USER_HANDLER not set in .env");
        let password = dotenvy::var("PASS").expect("PASS not set in .env");
        let employer_id = dotenvy::var("EMPLOYER_ID").expect("EMPLOYER_ID not set in .env");

        let mut client = Client::new();
        if let Err(e) = client.login(&username, &password).await {
            panic!("Login failed: {:?}", e);
        }
        println!("Login successful, token acquired");

        let result = client.list_applications(employer_id).await;
        assert!(
            result.is_ok(),
            "list_applications failed: {:?}",
            result.err()
        );

        let applications = result.unwrap();
        println!("Applications: {}", applications);
    }

    #[tokio::test]
    async fn test_add_employee() {
        dotenvy::dotenv().ok();

        let username = dotenvy::var("USER_HANDLER").expect("USER_HANDLER not set in .env");
        let password = dotenvy::var("PASS").expect("PASS not set in .env");
        let employer_id = dotenvy::var("EMPLOYER_ID").expect("EMPLOYER_ID not set in .env");

        let mut client = Client::new();
        if let Err(e) = client.login(&username, &password).await {
            panic!("Login failed: {:?}", e);
        }
        println!("Login successful, token acquired");
        let employee = AddEmployeeRequest::new(
            "Jesus".to_string(),
            "Christ".to_string(),
            "aaa 111111".to_string(),
            "UA".to_string(),
            "PL".to_string(),
            "11111111111".to_string(),
            "0000-12-25".to_string(),
        );
        let result = client.add_employee(employer_id, employee).await;
        assert!(result.is_ok(), "add_employee failed: {:?}", result.err());

        let response = result.unwrap();
        println!("Add employee response: {}", response);
    }
}
