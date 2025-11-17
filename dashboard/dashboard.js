async function login(url) {
    var username = document.getElementById("username").value;
    var password = document.getElementById("password").value;
    var login_request = {
        "username": username,
        "password": password,
    }
    let login_url = get_url(url);
    await send_request(login_url, "POST", JSON.stringify(login_request), true)
}

async function send(relative_url, element, post, drop_query) {
    let url = get_url(relative_url);
    if (!drop_query) url = add_admin_query(url);
    let type = "GET";
    if (post) { type = "POST" };
    let response = await send_request(url, type, "", true);
    if (element) {
        await add_response(response, element)
    }
}

async function change_password() {
    let new_password = document.getElementById("instance_pwd").value;
    let url = get_url("/admin/change_instance_password");
    url = add_admin_query(url);
    var change_request = {
        "password": new_password
    };
    let response = await send_request(url, "POST", JSON.stringify(change_request), true);
    await add_response(response, "")
}

async function add_instance() {
    let example_url = get_url("/admin/example");
    let example_response = await send_request(example_url);
    let user_json = await add_response(example_response, "return")

    user_json.email = document.getElementById("new_email").value;
    user_json.personeelsnummer = document.getElementById("new_psn").value;
    user_json.password = document.getElementById("new_pwd").value;
    let new_shift_mail = document.getElementById("new_shift").checked;
    let updated_shift_mail = document.getElementById("updated_shift").checked;
    user_json.user_properties.execution_interval_minutes = Number(document.getElementById("exec_int").value) * 60;
    user_json.user_properties.send_mail_new_shift = new_shift_mail;
    user_json.user_properties.send_mail_updated_shift = updated_shift_mail
    user_json.user_properties.send_mail_removed_shift = Boolean(new_shift_mail | updated_shift_mail);
    user_json.user_properties.send_failed_signin_mail = true
    user_json.user_properties.send_welcome_mail = true
    user_json.user_properties.split_night_shift = document.getElementById("split_midnight").checked;
    user_json.user_properties.stop_midnight_shift = document.getElementById("stop_midnight").checked;

    let user_url = get_url("/admin/add_instance")
    let response = await send_request(user_url, "POST", JSON.stringify(user_json))
    add_response(response)
}

async function add_response(response, element) {
    document.getElementById("response").style = ""
    // document.getElementById("response").innerText = "";
    if (response.status != 200) {
        let reponse_text = await response.text()
        if (reponse_text != "") document.getElementById("response").innerText = reponse_text;
        else document.getElementById("response").innerText = reponse.status
        document.getElementById("response").style = "color:red;"
    } else {
        if (element == "array") {
            document.getElementById("response").innerHTML = (await response.json()).join('\n')
        }
        else if (element == "return") return (await response.json());
        else if (element == "string") document.getElementById("response").innerText = await response.text()
        else if (element) {
            let response_json = await response.json()
            document.getElementById("response").innerText = response_json[element]
        }

    }
}

async function send_request(url, method, content) {
    let request = {
        method: method,
        headers: { 'Content-Type': 'application/json' },
    }
    if (content) request.body = content
    request.credentials = "include"
    let response = await fetch(url, request)
    return response
}

function get_url(addition) {
    return document.getElementById("url").value + addition;
}

function add_admin_query(url) {
    let name = document.getElementById("name").value;
    let email = document.getElementById("email").value;
    let account_name = document.getElementById("an").value;
    let instance_name = document.getElementById("psn").value;

    let params = {};
    if (name) params.name = name;
    if (email) params.email = email;
    if (account_name) params.account_name = account_name;
    if (instance_name) params.instance_name = instance_name;

    let query = new URLSearchParams(params);
    return (url + "?" + query)
}