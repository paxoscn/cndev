function onInit(user) {
    document.getElementById("button_to_post_adding_page").addEventListener('click', addPost);

    var user_json = localStorage.getItem('user');
    var user = "";
    eval("user = " + user_json);

    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                var posts = "";
                eval("posts = " + xhr.responseText);

                onPosts(user, posts);
            } else {
                console.log(xhr.status);
                showToast("");
            }
        }
    };

    xhr.open("GET", "https://www.sololo.cn/cndev/api/posts", true);

    xhr.setRequestHeader("Authorization", "Bearer " + user.token);

    xhr.send(null);
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