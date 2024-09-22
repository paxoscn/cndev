
function onInit(user) {
    if (user != null) {
        document.getElementById("content_main").style.display = "block";
    } else {
        window.location.href = "/";
    }

    document.getElementById("logout_button").addEventListener('click', logOut);
    document.getElementById("back_button").addEventListener('click', goBack);
}

function logOut() {
    localStorage.removeItem("user");

    window.location.href = "/";
}

function goBack() {
    window.history.back();
}