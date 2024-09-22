var currentHour = new Date().getHours();
var period = currentHour >= 7 && currentHour < 19 ? "day" : "night";
loadStylesheet("https://www.sololo.cn/cndev/css/" + document.getElementsByTagName("HTML")[0].getAttribute("template") + "-" + period + ".css");

function sendCode() {
    var tel = document.getElementById("tel").value;
    var time = new Date().toString().replace(/.* ([\d:]+) .*/, "$1");
    if (tel.length < 1) {
        showLog([
            "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_warn\">WARN</span> ValidationException: <span class=\"log_exception_message\">手机号(phoneNumber)不能为空</span>",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.validateInputs()",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.send()"
        ]);
    } else if (tel.length != 11) {
        showLog([
            "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_warn\">WARN</span> ValidationException: <span class=\"log_exception_message\">手机号(phoneNumber)必须为11位</span>",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.validateInputs()",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.send()"
        ]);
    } else {
        showLog([
            "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_info\">INFO</span> <span class=\"log_normal_message\">正在发送验证码...</span>",
            "&#160;",
            "&#160;"
        ]);

        const xhr = new XMLHttpRequest();

        xhr.onreadystatechange = function() {
            if (xhr.readyState === 4) {
                if (xhr.status === 200) {
                    showLog([
                        "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_info\">INFO</span> <span class=\"log_normal_message\">验证码发送成功</span>",
                        "&#160;"
                    ]);
                } else {
                    showLog([
                        "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_warn\">WARN</span> InternalException: <span class=\"log_exception_message\">发送验证码失败(" + xhr.status + "), 请稍后尝试</span>",
                        "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.send()"
                    ]);
                }
            }
        };

        xhr.open("POST", "https://www.sololo.cn/cndev/api/users/commands/sms-sending", true);
        xhr.setRequestHeader("Content-Type", "application/json");

        xhr.send(JSON.stringify({ "tel": tel }));
    }
}

function verifyCode() {
    var tel = document.getElementById("tel").value;
    var code = document.getElementById("code").value;
    var time = new Date().toString().replace(/.* ([\d:]+) .*/, "$1");
    if (tel.length < 1) {
        showLog([
            "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_warn\">WARN</span> ValidationException: <span class=\"log_exception_message\">手机号(phoneNumber)不能为空</span>",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.validateInputs()",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.verify()"
        ]);
    } else if (tel.length != 11) {
        showLog([
            "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_warn\">WARN</span> ValidationException: <span class=\"log_exception_message\">手机号(phoneNumber)必须为11位</span>",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.validateInputs()",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.verify()"
        ]);
    } else if (code.length < 1) {
        showLog([
            "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_warn\">WARN</span> ValidationException: <span class=\"log_exception_message\">验证码(code)不能为空</span>",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.validateInputs()",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.verify()"
        ]);
    } else if (code.length != 6) {
        showLog([
            "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_warn\">WARN</span> ValidationException: <span class=\"log_exception_message\">验证码(code)必须为6位</span>",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.validateInputs()",
            "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.verify()"
        ]);
    } else {
        showLog([
            "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_info\">INFO</span> <span class=\"log_normal_message\">正在验证...</span>",
            "&#160;",
            "&#160;"
        ]);

        const xhr = new XMLHttpRequest();

        xhr.onreadystatechange = function() {
            if (xhr.readyState === 4) {
                if (xhr.status === 201) {
                    showLog([
                        "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_info\">INFO</span> <span class=\"log_normal_message\">验证成功</span>",
                        "&#160;"
                    ]);

                    var user = "";
                    eval("user = " + xhr.responseText);
                    localStorage.setItem("user", xhr.responseText);
                    
                    window.location.reload();
                } else {
                    showLog([
                        "<span class=\"event_time\">[" + time + "]</span> <span class=\"log_warn\">WARN</span> InternalException: <span class=\"log_exception_message\">验证失败(" + xhr.status + "), 请稍后尝试</span>",
                        "&#160;&#160;&#160;&#160;&#160;&#160;&#160;&#160;at CN.dev.verify()"
                    ]);
                }
            }
        };

        xhr.open("POST", "https://www.sololo.cn/cndev/api/tokens", true);
        xhr.setRequestHeader("Content-Type", "application/json");

        xhr.send(JSON.stringify({ "tel": tel, "sms_code": code }));
    }
}

function showLog(lines) {
    document.getElementById("outputs").style.visibility = "visible";

    if (lines.length >= 3) document.getElementById("output_line_1").innerHTML = lines[lines.length - 3];
    if (lines.length >= 2) document.getElementById("output_line_2").innerHTML = lines[lines.length - 2];
    if (lines.length >= 1) document.getElementById("output_line_3").innerHTML = lines[lines.length - 1];
}

function loadStylesheet(url) {
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = url;
    link.onload = function() { console.log("CSS file " + url + " loaded"); };
    link.onerror = function() { console.log("Failed to load CSS file " + url); };
    document.head.appendChild(link);
}

function logIn() {
    document.getElementById("content_for_logging_in").style.display = "block";
    document.getElementById("content_main").style.display = "none";
}

function goToSettings() {
    window.location.href = "/settings";
}

window.addEventListener('load', function () {
    var user = null;
    var user_json = localStorage.getItem('user');
    if (user_json != null) {
        eval("user = " + user_json);
        document.querySelectorAll("#settings_div").forEach((div) => {
            div.style.display = "block";
        });
    } else {
        document.querySelectorAll("#login_div").forEach((div) => {
            div.style.display = "block";
        });
    }

    document.querySelectorAll(".input").forEach((input) => {
        input.addEventListener('keyup', function (e) {
            var input = e.target;
            if (e.keyCode === 13) {
                input.blur();
                if (input.getAttribute("id") == "tel") {
                    sendCode();
                } else {
                    verifyCode();
                }
            } else {
                input.setAttribute("size", input.value.length + 1);
                input.setAttribute("maxlength", input.value.length + 1);
            }
        });
    });

    document.getElementById("button_sending").addEventListener('click', sendCode);
    document.getElementById("button_verifying").addEventListener('click', verifyCode);

    document.querySelectorAll("#settings_button").forEach((button) => {
        button.addEventListener('click', goToSettings);
    });

    document.querySelectorAll("#login_button").forEach((button) => {
        button.addEventListener('click', logIn);
    });

    onInit(user);
});