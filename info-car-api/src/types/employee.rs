use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddEmployeeRequest {
    pub person: Person,
    pub contact_details: ContactDetails,
    pub technician_details_dto: Option<Value>,
    pub control_officer_details_dto: Option<Value>,
    pub employee_type: i32,
    pub driver_details_dto: DriverDetails,
}

impl AddEmployeeRequest {
    pub fn new(
        first_name: String,
        last_name: String,
        driving_license_serial_number: String,
        driving_license_authority_country: String,
        residence_country_code: String,
        pesel: String,
        birth_date: String,
    ) -> Self {
        Self {
            person: Person {
                first_name,
                last_name,
                pesel,
                ..Default::default()
            },
            contact_details: ContactDetails {
                residence_address: Some(ResidenceAddress {
                    country_code: residence_country_code,
                    ..Default::default()
                }),
                ..Default::default()
            },
            driver_details_dto: DriverDetails {
                birth_date,
                driving_license_dto: DrivingLicense {
                    serial_number: driving_license_serial_number,
                    authority_country: driving_license_authority_country,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub pesel: String,
    pub has_no_pesel: Option<bool>,
    pub personal_document_type: Option<String>,
    pub personal_document_number: Option<String>,
    pub personal_document_issuing_authority: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactDetails {
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub residence_address: Option<ResidenceAddress>,
    pub mailing_address: Option<Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidenceAddress {
    pub country_code: String,
    pub street: Option<String>,
    pub building_no: Option<String>,
    pub apartment_no: Option<String>,
    pub city: Option<String>,
    pub post_code: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverDetails {
    pub birth_date: String,
    pub birth_place: Option<String>,
    pub driver_card_number: Option<String>,
    pub foreign_card_expiry_date: Option<String>,
    pub foreign_issue_country: Option<String>,
    pub driving_license_dto: DrivingLicense,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrivingLicense {
    pub authority_country: String,
    pub release_date: Option<String>,
    pub expiration_date: Option<String>,
    pub document_number: Option<String>,
    pub serial_number: String,
    pub issuing_authority: Option<String>,
    pub valid_indefinitely: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddEmployeeSuccess {
    pub id: String,
}
