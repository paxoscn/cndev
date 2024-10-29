
const STATUS_DRAFT = 1;
const STATUS_PUBLISHED = 2;
const STATUS_DELETED = 3;

function onInit(user) {
    document.getElementById("content_main").style.display = "block";

    if (author_id.length < 1) {
        fetchAuthorByNick(user);
    } else {
        onAuthor(user);
    }
}

function fetchAuthorByNick(user) {
    const xhr = new XMLHttpRequest();
        
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                var userRes = "";
                eval("userRes = " + xhr.responseText);

                author_id = userRes.id;
                author_registering_time = new Date(userRes.created_at).getTime() / 1000;

                onAuthor(user);
            } else {
                console.log(xhr.status);
                showToast("");
            }
        }
    };

    xhr.open("GET", "https://www.sololo.cn/cndev/api/users/" + author_nick, true);

    xhr.setRequestHeader("Authorization", "Bearer " + user.token);

    xhr.send(null);
}

function onAuthor(user) {
    tryApplyAvatar(author_id);

    var urlSuffix = window.location.href.replace(/.*\//, "");
    if (!/^\d+$/.test(urlSuffix)) {
        var nick = urlSuffix;
        document.getElementById("nick").innerHTML = nick;
    }

    document.getElementById("joined_days").innerHTML = Math.ceil((Date.now() - author_registering_time * 1000 + 1) / 86400000);
    
    if (user != null && user.id === author_id) {
        document.querySelector("legend").style.display = "block";
        
        document.querySelectorAll(".post_panel button").forEach((el) => {
            el.style.visibility = "visible";
        });

        document.getElementById("button_to_post_adding_page").addEventListener('click', addPost);
    
        const xhr = new XMLHttpRequest();
    
        xhr.onreadystatechange = function() {
            if (xhr.readyState === 4) {
                if (xhr.status === 200) {
                    var postsRes = "";
                    eval("postsRes = " + xhr.responseText);
    
                    onDraftPosts(user, postsRes);
                } else {
                    console.log(xhr.status);
                    showToast("");
                }
            }
        };
    
        xhr.open("GET", "https://www.sololo.cn/cndev/api/posts?status=" + STATUS_DRAFT, true);
    
        xhr.setRequestHeader("Authorization", "Bearer " + user.token);
    
        xhr.send(null);
    }
}

function onDraftPosts(user, postsRes) {
    var draftPosts = postsRes.entities;
    var draftPostsHtml = "";
    draftPosts.forEach((draftPost) => {
        draftPostsHtml += `
<div class="post" _post_id="` + draftPost.id + `" _post_author_id="` + user.id + `" _post_author_nick="` + user.nick + `" _post_sharing_path="` + draftPost.sharing_path + `" _post_status="` + draftPost.status + `">
    <div class="post_title"><span>` + draftPost.title + `</span></div>
    <div class="post_panel">
        <span class="post_updating"><button class="post_updating_button" disabled="true">编辑</button></span>
        <span class="post_publishing"><button class="post_publishing_button" disabled="true">发表</button></span>
        <span class="post_unpublishing"><button class="post_unpublishing_button" disabled="true">撤销</button></span>
        <span class="post_deleting"><button class="post_deleting_button" disabled="true">删除</button></span>
    </div>
    <div class="post_time">` + draftPost.updated_at_formatted + `</div>
</div>
        `;
    });
    document.getElementById("posts").innerHTML = draftPostsHtml + document.getElementById("posts").innerHTML;

    document.querySelectorAll(".post_panel button").forEach((el) => {
        el.style.visibility = "visible";
    });

    document.querySelectorAll(".post").forEach((el) => {
        el.querySelector(".post_title").addEventListener('click', function (e) {
            var el = e.target;
            while (el.className !== "post") el = el.parentNode;
            window.location.href =
                    "/" + (el.getAttribute("_post_author_nick").length < 1 ? el.getAttribute("_post_author_id") : el.getAttribute("_post_author_nick")) +
                    "/" + (el.getAttribute("_post_sharing_path").length < 1 ? el.getAttribute("_post_id") : el.getAttribute("_post_sharing_path"));
        });

        if (user != null && user.id === parseInt(el.getAttribute("_post_author_id"))) {
            el.querySelector(".post_updating_button").disabled = false;
            switch (parseInt(el.getAttribute("_post_status"))) {
                case STATUS_DRAFT:
                    el.querySelector(".post_publishing_button").disabled = false;
                    break;
                case STATUS_PUBLISHED:
                    el.querySelector(".post_unpublishing_button").disabled = false;
                    break;
            }
            el.querySelector(".post_deleting_button").disabled = false;
            el.querySelector(".post_updating_button").addEventListener('click', function (e) {
                var el = e.target;
                while (el.className !== "post") el = el.parentNode;
                window.location.href = "/posts/" + el.getAttribute("_post_id");
            });
            el.querySelector(".post_publishing_button").addEventListener('click', function (e) {
                var el = e.target;
                while (el.className !== "post") el = el.parentNode;

                if (confirm("是否发表 " + el.querySelector(".post_title > span").innerHTML + " ?")) {
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
            el.querySelector(".post_unpublishing_button").addEventListener('click', function (e) {
                var el = e.target;
                while (el.className !== "post") el = el.parentNode;

                if (confirm("是否撤销 " + el.querySelector(".post_title > span").innerHTML + " ?")) {
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
                
                    xhr.open("PUT", "https://www.sololo.cn/cndev/api/posts/" + el.getAttribute("_post_id") + "/unpublishing", true);
                
                    var user_json = localStorage.getItem('user');
                    var user = "";
                    eval("user = " + user_json);
                
                    xhr.setRequestHeader("Authorization", "Bearer " + user.token);
                
                    xhr.send(null);
                }
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
        }
    });
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