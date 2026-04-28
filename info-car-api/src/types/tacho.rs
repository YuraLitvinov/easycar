use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// --- Lookup types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub country_code: String,
    pub country_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Countries {
    pub countries: Vec<Country>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistrictOffice {
    pub id: String,
    pub code: String,
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistrictOffices {
    pub district_offices: Vec<DistrictOffice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersonalDocumentType {
    Passport,
    IdentityCard,
    PermanentResidenceCard,
    PolesCard,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonDocumentTypeEntry {
    pub personal_document_type: PersonalDocumentType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonDocumentTypes {
    pub values: Vec<PersonDocumentTypeEntry>,
    pub count: u32,
}

// --- Application list (card order tracking) ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TachoApplicationList {
    pub applications: Vec<TachoApplication>,
    pub count: u32,
    pub total_pages: u32,
    pub total_count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TachoApplication {
    pub id: Option<String>,
    pub status: Option<String>,
}

pub enum TachoApplicationFilter {
    OnlyCount,
    Draft,
    OnlyCountDraft,
}

impl TachoApplicationFilter {
    pub fn as_query_value(&self) -> &'static str {
        match self {
            Self::OnlyCount => "onlyCount",
            Self::Draft => "draft",
            Self::OnlyCountDraft => "onlyCount,draft",
        }
    }
}

// --- Employee types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonDto {
    pub id: Option<String>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub pesel: Option<String>,
    pub has_no_pesel: Option<bool>,
    pub personal_document_type: Option<PersonalDocumentType>,
    pub personal_document_number: Option<String>,
    pub personal_document_issuing_authority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressDto {
    pub id: Option<String>,
    pub country_code: String,
    pub street: Option<String>,
    pub building_no: Option<String>,
    pub apartment_no: Option<String>,
    pub city: Option<String>,
    pub post_code: Option<String>,
    pub company_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactDetailsDto {
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub residence_address: AddressDto,
    pub mailing_address: Option<AddressDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrivingLicenseDto {
    pub id: Option<String>,
    pub authority_country: String,
    pub release_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub document_number: Option<String>,
    pub serial_number: Option<String>,
    pub issuing_authority: Option<String>,
    pub valid_indefinitely: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverDetailsDto {
    pub birth_date: NaiveDate,
    pub birth_place: Option<String>,
    pub driver_card_number: Option<String>,
    pub foreign_card_expiry_date: Option<NaiveDate>,
    pub foreign_issue_country: Option<String>,
    pub driving_license_dto: DrivingLicenseDto,
}

/// Request body for `POST /api/stc/employers/{id}/employees/`.
/// `employee_type`: 0 = DRIVER
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewEmployee {
    pub person: PersonDto,
    pub contact_details: ContactDetailsDto,
    pub technician_details_dto: Option<()>,
    pub control_officer_details_dto: Option<()>,
    pub employee_type: u8,
    pub driver_details_dto: Option<DriverDetailsDto>,
}

impl NewEmployee {
    pub fn new_driver(
        person: PersonDto,
        contact_details: ContactDetailsDto,
        driver_details: DriverDetailsDto,
    ) -> Self {
        Self {
            person,
            contact_details,
            technician_details_dto: None,
            control_officer_details_dto: None,
            employee_type: 0,
            driver_details_dto: Some(driver_details),
        }
    }
}

/// Response body from `POST /api/stc/employers/{id}/employees/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Employee {
    pub id: String,
    pub employee_type: String,
    pub person: PersonDto,
    pub contact_details: ContactDetailsDto,
    pub driver_details_dto: Option<DriverDetailsDto>,
}

/// Entry in `GET /api/stc/employers/{id}/employees/search` results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeSummary {
    pub id: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub deleted: bool,
    pub driver_card_number: Option<String>,
    pub foreign_card_expiry_date: Option<NaiveDate>,
    pub foreign_issue_country: Option<String>,
    pub technician_card_number: Option<String>,
    pub control_card_number: Option<String>,
    pub error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeSearchResult {
    pub employees: Vec<EmployeeSummary>,
    pub count: u32,
    pub total_pages: u32,
    pub total_count: u32,
}
