function login() {
    var username = document.getElementById("username").value;
    var password = document.getElementById("password").value;
    var login_request = {
        "username": username,
        "password": password,
    }
    let login_url = get_url("/login");
    send_request(login_url, "POST", JSON.stringify(login_request), true)
}

function logout() {
    let url = get_url("/logout");
    send_request(url, "GET", "", true);
}

async function send(relative_url, element, post) {
    let url = get_url(relative_url);
    url = add_admin_query(url);
    let type = "GET";
    if (post) { "POST" };
    let response = await send_request(url, type, "", true);
    if (element) {
        await add_response(response, element)
    }

}


async function change_password() {
    let new_password = document.getElementById("instance_pwd").value;
    let url = get_url("/admin/change_instance_password");
    var change_request = {
        "password": new_password
    };
    let response = await send_request(url, "POST", JSON.stringify(change_request), true);
    await add_response(response, "")
}

async function add_response(response, element) {
    document.getElementById("response").style = ""
    if (response.status != 200) {
        document.getElementById("response").innerText = response.status;
        document.getElementById("response").style = "color:red;"
    } else {
        if (element) {
            let response_json = await response.json()
            document.getElementById("response").innerText = response_json[element]
        }
        else if (element == "return") return await response.text();
        else document.getElementById("response").innerText = await response.text()
    }
}

async function send_request(url, method, content, credentials) {
    let request = {
        method: method,
        headers: { 'Content-Type': 'application/json' },
    }
    if (content) request.body = content
    if (credentials == true) request.credentials = "include"
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