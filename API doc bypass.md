`API versie 1`
---

# Alle (user) endpoints

## Account management

| Method | Endpoint                  | Datatype           | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| ------ | ------------------------- | ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| POST   | `/bypass/change_password` | PasswordChange     | Wijzigd het wachtwoord van de gebruiker (Logt de gebruiker uit)                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| PUT    | `/bypass/instance`        | MijnBussieInstance | Voegt een nieuwe user toe aan het systeem. Bij _OK_ bevat de body de user name waarmee je met het get request de status van de user kan opvragen. Bij _FOUND (302)_ bestaat de user al, in de body is de calendar link van deze user. Bij _NOT_ACCEPTABLE (406)_ bestaat de user waarschijnlijk al maar met een ander email, het account is dan niet aangemaakt                                                                                                                                          |
| GET    | `/bypass/instance/{name}` |                    | Returnt de huidige staat van deze instance. Bij _ACCEPTED (202)_ zijn de inloggegevens correct maar is er nog geen data op de calendar link. Bij _OK (200)_ is er data bij de calendar link. Bij _TOO_EARLY (425)_ is het nog niet duidelijk of de credentials correct zijn. Bij _NOT_ACCEPTABLE (406)_ waren de inloggegevens incorrect. De instance is daarom verwijderd. Je kan gewoon een nieuwe instance aanmaken. De calendar link is te door het formulier met **PUT** nog een keer op te sturen. |


# Datatypes

## PasswordChange

``` JSON
{
    "calendar_link": "Link",
    "personeelsnummer": "Personeelsnummer",
    "password": "Password"
}
```

# MijnBussieInstance
``` JSON
{
    "personeelsnummer": "123456",
    "password": "Password",
    "email": "youp@protonmail.com",
    "is_student": true
    {
        "send_mail_new_shift": false,
        "send_mail_updated_shift": false,
        "send_mail_removed_shift": false,
        "send_failed_signin_mail": false,
        "split_night_shift": false,
        "stop_midnight_shift": false
    }

}
    
```