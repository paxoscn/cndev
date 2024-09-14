function onInit(user) {
    document.getElementById("button_to_post_adding_page").addEventListener('click', addPost);

    document.querySelectorAll(".post").forEach((el) => {
        el.addEventListener('click', function (e) {
            var el = e.target;
            while (el.className !== "post") el = el.parentNode;
            window.location.href =
                    "/" + (el.getAttribute("_post_author_nick").length < 1 ? el.getAttribute("_post_author_id") : el.getAttribute("_post_author_nick")) +
                    "/" + (el.getAttribute("_post_sharing_path").length < 1 ? el.getAttribute("_post_id") : el.getAttribute("_post_sharing_path"));
        });
    });
}

function onPosts(user, posts) {
    console.log(posts);
}

function addPost() {
    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 201) {
                var post = "";
                eval("post = " + xhr.responseText);

                window.location.href = "/posts/" + post.id;
            } else if (xhr.status === 401) {
                localStorage.removeItem('user')

                window.location.reload();
            } else {
                showToast("");
            }
        }
    };

    xhr.open("POST", "https://www.sololo.cn/cndev/api/posts", true);
    xhr.setRequestHeader("Content-Type", "application/json");

    var user_json = localStorage.getItem('user');
    var user = "";
    eval("user = " + user_json);

    xhr.setRequestHeader("Authorization", "Bearer " + user.token);

    xhr.send(JSON.stringify({ "title": "(草稿)", "sharing_path": "", "tags": "", "text": "" }));
}

function showToast(message) {
    const toastElement = document.createElement('div');
    toastElement.className = 'toast';
    toastElement.textContent = message;

    document.body.appendChild(toastElement);

    setTimeout(() => {
      document.body.removeChild(toastElement);
    }, 3000);
  }