function onInit(user) {
    if (user != null) {
        var last_page = localStorage.getItem('lastPage');
        if (last_page != null) {
            localStorage.removeItem('lastPage');
            window.location.href = last_page;
        } else {
            window.location.href = "/" + (user.nick.length > 0 ? user.nick : user.id);
        }
    } else {
        document.getElementById("content_for_logging_in").style.display = "block";
    }
}