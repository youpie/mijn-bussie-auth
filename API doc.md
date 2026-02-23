`API versie 1`
---

# Alle (user) endpoints

## Account management

| Method | Endpoint           | Datatype       | Description                                                     |
| ------ | ------------------ | -------------- | --------------------------------------------------------------- |
| POST   | `/login`           | Credentials    | Logt een gebruikter in                                          |
| GET    | `/logout`          |                | Logt een gebruiker uit                                          |
| POST   | `/signup`          | Credentials    | Maakt een nieuw account aan                                     |
| GET    | `/me`              |                | Toont de huidig ingelogde gebruiker                             |
| POST   | `/change_password` | PasswordChange | Wijzigd het wachtwoord van de gebruiker (Logt de gebruiker uit) |
| GET    | `/role`            |                | Stuurt de huidige rol (permissies) van de gebruiker             |

## Instance management
_Beheren van instance gegevens_
| Method | Endpoint                       | Datatype            | Description                                                                            |
| ------ | ------------------------------ | ------------------- | -------------------------------------------------------------------------------------- |
| POST   | `/change_instance_information` | InstanceInformation | Verander de email en wachtwoord van de mijnBussie instance verbonden met het account   |
| POST   | `/add_instance`                | MijnBussieInstance  | Creert een nieuwe MijnBussie instance en linkt automatisch de instance aan het account |
| GET   | `/calendar` | | Stuurt de agenda link van de gelinkte instance |
| GET | `/ExitCode` | | Stuurt de laatste exitcode van de gelinkte instance terug
| POST | `/Delete` | | Verwijderd de Instance van deze user |

# Datatypes

## Credentials

``` JSON
{
    "username": "youpie",
    "password": "Password"
}
```

## PasswordChange

``` JSON
{
    "password": "Password"
}
```

## InstanceInformation 
_Alle waardes zijn optioneel_
``` JSON
{
    "email": "youp@protonmail.com",
    "password": "Password",
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