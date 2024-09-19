function onInit(user) {
    if (user != null) {
        window.location.href = "/" + (user.nick.length > 0 ? user.nick : user.id);
    } else {
        document.getElementById("content_for_logging_in").style.display = "block";
    }
}