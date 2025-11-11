# User Endpoints
* /ChangePassword
* /DeleteAccount
    Dit moet dan ook vanuit Backend behandeld worden
* /DisableAccount

__/Instance__
* /calendar **\***
    Of: een redirect naar calendar.bussie.app
    Of: Een static file host in Rust (misschien sneller)  ✅
* /Logbook ✅
* /Start **\***
    Rate limiten (per user natuurlijk)
* /IsActive **\***
* /ExitCode **\***
* /PasswordChange **\***
    Sterk Rate limiten (ook per user)
* /AddInstance **\***
* /ChangeEmail
* /PropertiesChange
    Beschermde kolommen niet updaten
    Misschien ooit met bevestiging?
* /GetProperties **?**
    Zodat bussie dynamisch dingen kan toevoegen
* /DataChange
    Same Same


* /Add -> /SignUp
    Hier moet (misschien) een captcha challenge en rate limit aan toegevoegd worden
    
    
* /Login
    Rate limiten


# Admin Endpoints
* /Json/{NewInstance, UserAccount}
    Zodat ik dat niet hoef te onthouden (is dit makkelijk te genereren met SerdeJson?)
* /SiteMap
    
* /Instance/Update **!**
    Los voor email, en los voor user_properties
* /Instance/Get
    Zorgen dat hier gevoelige data uit wordt verwijderd
* /Instance/AddFromPath **!**
    Range Beperken
* /Instance/Failed **!**
* /Instance/Active

* /User/Get
    Idem Dito
* /User/Ban
* /User/Unban
* /ChangePassword **!**
    Naar /user en /instance

## Query
?user_name eerst
?instance_name daarna zoeken

# Extra
* Betere route logging
* Tracing toevoegen voor loggen
* Betere layers verwerken zoals in Youtube video
* HTTPS tussen alle exposed porten
* Betere error logging
* Role validation verbeteren
* Ansible en jeanette scripts updaten
