# Open routes
* /login
* /logout
* /signup


# Protected routes
## get
* /logbook
* /is_active
* /exit_code
* /name
* /calendar

## post
* /start <- Nog niet
* /add_instance
* /change_password

# Admin Routes
**alles /admin/**
## get
* /names
* /emails
* /logbook ?query
* /is_active ?query
* /exit_code ?query
* /name ?query
* /calendar ?query

* /get_instance ?query
* /example

* /instances
* /users
* /import_user {"path"}



## post
* /kuma/reset ?query
* /kuma/remove ?query
* /start ?query
* /refresh ?query

* /add_instance
* /change_instance_password
* /assign_instance ?query (waar instance_name = instance en account_name = account om aan te verbinden)
* /update_properties ?query


## Admin Query
- instance_name
- account_name
- name
- email
