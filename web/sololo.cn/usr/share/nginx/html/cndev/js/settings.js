
function onInit(user) {
    if (user != null) {
        document.getElementById("content_main").style.display = "block";
    } else {
        window.location.href = "/";
    }

    var user_json = localStorage.getItem('user');
    var user = "";
    eval("user = " + user_json);

    tryApplyAvatar(user.id, (has_avatar) => {
        if (has_avatar) {
            document.getElementById("avatar_button").innerHTML = "删除头像";
            document.getElementById("avatar_button").addEventListener('click', removeAvatar);
        } else {
            document.getElementById("avatar_button").addEventListener('click', openFileInput);
        }
    });

    document.getElementById("nick").value = user.nick;

    document.getElementById("avatar_input").addEventListener('change', uploadAvatar);

    document.getElementById("avatar").addEventListener('click', openFileInput);

    document.getElementById("nick_button").addEventListener('click', changeNick);
    document.getElementById("logout_button").addEventListener('click', logOut);
    document.getElementById("back_button").addEventListener('click', goBack);
}

function openFileInput() {
    document.getElementById("avatar_input").click();
}

function uploadAvatar() {
    if (document.getElementById("avatar_input").files.length < 1) {
        if (confirm("要删除头像吗?")) {
            removeAvatar();
        }

        return;
    }

    var file = document.getElementById("avatar_input").files[0];
    var fileName = file.name;
    var fileSize = file.size;console.log(fileSize);

    var validExtensions = ["png", "jpg", "jpeg", "gif", "webp"];
    var fileExtension = fileName.split('.').pop().toLowerCase();

    if (fileSize < 1 || fileSize > 1024 * 100) {
        alert("文件大小必须在100KB以内");
        return;
    }

    if (!validExtensions.includes(fileExtension)) {
        alert("无效的文件扩展名。请上传png, jpg, jpeg, gif或webp格式的文件");
        return;
    }

    var user_json = localStorage.getItem('user');
    var user = "";
    eval("user = " + user_json);

    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                var res = "";
                eval("res = " + xhr.responseText);

                document.getElementById("avatar").src = "https://www.sololo.cn/cndev/_avatars/" + res.uploaded_file_name;
            } else if (xhr.status === 401) {
                localStorage.removeItem('user')

                window.location.reload();
            } else {
                showToast("");
            }
        }
    };

    xhr.open("PUT", "https://www.sololo.cn/cndev/api/settings/avatar", true);

    xhr.setRequestHeader("Authorization", "Bearer " + user.token);

    const formData = new FormData();
    formData.append("avatar", document.getElementById("avatar_input").files[0]);

    xhr.send(formData);
}

function removeAvatar() {
    var user_json = localStorage.getItem('user');
    var user = "";
    eval("user = " + user_json);

    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                window.location.reload();
            } else if (xhr.status === 401) {
                localStorage.removeItem('user')

                window.location.reload();
            } else {
                showToast("");
            }
        }
    };

    xhr.open("DELETE", "https://www.sololo.cn/cndev/api/settings/avatar", true);

    xhr.setRequestHeader("Authorization", "Bearer " + user.token);

    xhr.send(null);
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