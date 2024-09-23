
function onInit(user) {
    if (user != null) {
        document.getElementById("content_main").style.display = "block";
    } else {
        window.location.href = "/";
    }

    document.getElementById("nick").value = user.nick;

    document.getElementById("nick_button").addEventListener('click', changeNick);
    document.getElementById("logout_button").addEventListener('click', logOut);
    document.getElementById("back_button").addEventListener('click', goBack);
}

function changeNick() {
    var user_json = localStorage.getItem('user');
    var user = "";
    eval("user = " + user_json);

    var nick = document.getElementById("nick").value;
    if (nick === user.nick) {
        return;
    }

    var prompt = nick.length > 0 ? ("要更改昵称为 " + nick + " 吗?") : ("要删除昵称吗?");

    if (!confirm(prompt)) {
        return;
    }

    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                var user = "";
                eval("user = " + xhr.responseText);
                localStorage.setItem("user", xhr.responseText);

                window.location.reload();
            } else if (xhr.status === 401) {
                localStorage.removeItem('user')

                window.location.reload();
            } else {
                showToast("");
            }
        }
    };

    xhr.open("PUT", "https://www.sololo.cn/cndev/api/settings", true);
    xhr.setRequestHeader("Content-Type", "application/json");

    var user_json = localStorage.getItem('user');
    var user = "";
    eval("user = " + user_json);

    xhr.setRequestHeader("Authorization", "Bearer " + user.token);

    xhr.send(JSON.stringify({ "nick": nick }));
}

function logOut() {
    localStorage.removeItem("user");

    window.location.href = "/";
}

function goBack() {
    window.history.back();
}