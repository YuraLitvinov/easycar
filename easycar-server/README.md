# Easycar Server

A standalone Rust backend for interacting with the info-car.pl Tachograph API. This server handles authentication and communicates with the info-car.pl STC (tachograph) endpoints.

## Configuration

The server requires the following environment variables (or provided via Vault):
- `USERNAME`: info-car.pl login
- `PASSWORD`: info-car.pl password
- `EMPLOYER_ID`: Your employer UUID on info-car.pl
- `PORT`: Server port (default: 3000)

## API Endpoints

### Headers
| Header | Requirement | Description |
|--------|-------------|-------------|
| `X-CF-Turnstile` | Optional | Cloudflare Turnstile token if required by info-car.pl |

---

### Employees

#### `GET /tachograph/employees`
Search for registered employees/drivers.
- **Query Parameters**:
  - `page`: Page number (default: 0)
  - `limit`: Items per page (default: 12)
- **Response**: `EmployeeSearchResult` JSON.

#### `POST /tachograph/employees`
Register a new driver.
- **Body**: `NewEmployee` JSON (see Data Structures below).
- **Response**: `201 Created` with the created `Employee` JSON.

#### `DELETE /tachograph/employees/:id`
Remove an employee by their UUID.
- **Response**: `204 No Content`.

---

### Applications

#### `GET /tachograph/applications`
List tachograph card applications.
- **Response**: `TachoApplicationList` JSON.

---

### Lookups

#### `GET /tachograph/countries`
Get a list of supported countries (basic).

#### `GET /tachograph/countries/aetr`
Get a list of AETR countries.

#### `GET /tachograph/district-offices`
Get a list of Polish district offices (Wydziały Komunikacji).

#### `GET /tachograph/document-types`
Get a list of supported personal document types (Passport, ID Card, etc.).

---

## Data Structures

### `NewEmployee` (Request Body)
```json
{
  "person": {
    "firstName": "String",
    "middleName": "String | null",
    "lastName": "String",
    "pesel": "String | null",
    "hasNoPesel": "boolean | null",
    "personalDocumentType": "PASSPORT | IDENTITY_CARD | ... | null",
    "personalDocumentNumber": "String | null",
    "personalDocumentIssuingAuthority": "String | null"
  },
  "contactDetails": {
    "email": "String | null",
    "phoneNumber": "String | null",
    "residenceAddress": {
      "countryCode": "PL",
      "street": "String | null",
      "buildingNo": "String | null",
      "apartmentNo": "String | null",
      "city": "String | null",
      "postCode": "String | null"
    },
    "mailingAddress": null
  },
  "employeeType": 0,
  "driverDetailsDto": {
    "birthDate": "YYYY-MM-DD",
    "birthPlace": "String | null",
    "drivingLicenseDto": {
      "authorityCountry": "UA",
      "serialNumber": "String | null",
      "validIndefinitely": "boolean | null"
    }
  }
}
```

### `Employee` (Response)
Matches the request structure but includes an `id` field for the employee and nested objects.

## Error Handling
The server returns `500 Internal Server Error` with a plain text description of the error if an API call fails. Specific API-level errors (validation issues) are returned as part of the error message.
