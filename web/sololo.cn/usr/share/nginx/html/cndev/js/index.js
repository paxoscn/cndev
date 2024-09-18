function onInit(user) {
    window.location.href = "/" + (user.nick.length > 0 ? user.nick : user.id);
}