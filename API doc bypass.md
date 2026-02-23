`API versie 1`
---

# Alle (user) endpoints

## Account management

| Method | Endpoint           | Datatype       | Description                                                     |
| ------ | ------------------ | -------------- | --------------------------------------------------------------- |

| POST   | `/bypass/change_password` | PasswordChange | Wijzigd het wachtwoord van de gebruiker (Logt de gebruiker uit) |


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
    {
        "send_mail_new_shift": false,
        "send_mail_updated_shift": false,
        "send_mail_removed_shift": false,
        "send_failed_signin_mail": false,
        "split_night_shift": false,
        "stop_midnight_shift": false,
    }

}
    
```