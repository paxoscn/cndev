function onInit(user) {
    document.getElementById("content_main").style.display = "block";
    
    document.getElementById("button_to_post_adding_page").addEventListener('click', addPost);

    document.querySelectorAll(".post").forEach((el) => {
        el.querySelector(".post_title").addEventListener('click', function (e) {
            var el = e.target;
            while (el.className !== "post") el = el.parentNode;
            window.location.href =
                    "/" + (el.getAttribute("_post_author_nick").length < 1 ? el.getAttribute("_post_author_id") : el.getAttribute("_post_author_nick")) +
                    "/" + (el.getAttribute("_post_sharing_path").length < 1 ? el.getAttribute("_post_id") : el.getAttribute("_post_sharing_path"));
        });

        if (user != null && user.id === parseInt(el.getAttribute("_post_author_id"))) {
            el.querySelector(".post_updating").style.visibility = "visible";
            el.querySelector(".post_deleting").style.visibility = "visible";
            if (el.getAttribute("_post_status") == "1") {
                el.querySelector(".post_publishing").style.visibility = "visible";
            }
            el.querySelector(".post_updating_button").addEventListener('click', function (e) {
                var el = e.target;
                while (el.className !== "post") el = el.parentNode;
                window.location.href = "/posts/" + el.getAttribute("_post_id");
            });
            el.querySelector(".post_deleting_button").addEventListener('click', function (e) {
                var el = e.target;
                while (el.className !== "post") el = el.parentNode;

                if (confirm("是否删除 " + el.querySelector(".post_title > span").innerHTML + " ?")) {
                    const xhr = new XMLHttpRequest();

                    xhr.onreadystatechange = function() {
                        if (xhr.readyState === 4) {
                            if (xhr.status === 200) {
                                window.location.reload();
                            } else if (xhr.status === 401) {
                                // TODO
                                localStorage.removeItem('user')
                
                                window.location.reload();
                            } else {
                                showToast("");
                            }
                        }
                    };
                
                    xhr.open("DELETE", "https://www.sololo.cn/cndev/api/posts/" + el.getAttribute("_post_id"), true);
                
                    var user_json = localStorage.getItem('user');
                    var user = "";
                    eval("user = " + user_json);
                
                    xhr.setRequestHeader("Authorization", "Bearer " + user.token);
                
                    xhr.send(null);
                }
            });
            el.querySelector(".post_publishing_button").addEventListener('click', function (e) {
                var el = e.target;
                while (el.className !== "post") el = el.parentNode;

                if (confirm("是否发布 " + el.querySelector(".post_title > span").innerHTML + " ?")) {
                    const xhr = new XMLHttpRequest();

                    xhr.onreadystatechange = function() {
                        if (xhr.readyState === 4) {
                            if (xhr.status === 200) {
                                window.location.reload();
                            } else if (xhr.status === 401) {
                                // TODO
                                localStorage.removeItem('user')
                
                                window.location.reload();
                            } else {
                                showToast("");
                            }
                        }
                    };
                
                    xhr.open("PUT", "https://www.sololo.cn/cndev/api/posts/" + el.getAttribute("_post_id") + "/publishing", true);
                
                    var user_json = localStorage.getItem('user');
                    var user = "";
                    eval("user = " + user_json);
                
                    xhr.setRequestHeader("Authorization", "Bearer " + user.token);
                
                    xhr.send(null);
                }
            });
        }
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