var currentHour = new Date().getHours();
var period = currentHour >= 7 && currentHour < 19 ? "day" : "night";
loadStylesheet("https://www.sololo.cn/cndev/css/" + document.getElementsByTagName("HTML")[0].getAttribute("id") + "-" + period + ".css");

import mermaid from 'https://www.sololo.cn/cndev/js/mermaid/mermaid.esm.min.js';

mermaid.initialize({ startOnLoad: false });

var simplemde = null;

function onInit(user) {
    if (user != null) {
        document.getElementById("content_main").style.display = "block";
    } else {
        window.location.href = "/";
    }
}

function onPost(user, post) {
    document.getElementById("button_saving").addEventListener('click', savePost);
    document.getElementById("button_cancelling").addEventListener('click', goBack);
    document.getElementById("button_url_copying").addEventListener('click', copyUrl);
    document.getElementById("button_path_generating").addEventListener('click', generatePath);
    document.getElementById("id").value = post.id;
    document.getElementById("title").value = post.title;
    document.getElementById("sharing_url_prefix").innerHTML = "https://cn.dev/" + (user.nick.length > 0 ? user.nick : user.id) + "/";
    document.getElementById("sharing_path").value = post.sharing_path;
    document.getElementById("tags").value = post.tags;
    document.getElementById("text").value = post.text;

    simplemde = new SimpleMDE({
        element: document.getElementById("text"),
        autosave: {
            enabled: true,
            uniqueId: "post-" + post.id,
            delay: 10000,
        },
        renderingConfig: {
            codeSyntaxHighlighting: true
        },
        previewRender: function(plainText, preview) {
            // Adding Mermaid support.
            /*
Test case:

# nnn

```java
public class Aaa {}
```

ccc

```mermaid
graph LR
A --- B
B-->C[fa:fa-ban forbidden]
B-->D(fa:fa-spinner)
```

vvv
            */
            var rendered = this.parent.markdown(plainText);

            setTimeout(function() {
                document.querySelectorAll(".lang-mermaid").forEach((element) => {
                    element.innerHTML = element.innerHTML.replace(/<span[^>]*>/g, "").replace(/<\/span>/g, "");
                })

                mermaid.run({
                    nodes: document.querySelectorAll('.lang-mermaid'),
                });
            }, 250);
    
            return rendered;
        },
    });
}

function savePost() {
    var user_json = localStorage.getItem('user');
    var user = "";
    eval("user = " + user_json);

    var id = document.getElementById("id").value;

    var post = {};
    post.title = document.getElementById("title").value;
    post.sharing_path = document.getElementById("sharing_path").value;
    post.tags = document.getElementById("tags").value;
    post.text = simplemde.value();

    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                simplemde.clearAutosavedValue();

                window.location.href = "/" + (user.nick.length > 0 ? user.nick : user.id);
            } else if (xhr.status === 403) {
                alert("保存失败，请检查内容后再尝试。");
            } else if (xhr.status === 401) {
                localStorage.removeItem('user')

                window.location.href = "/";
            } else {
                showToast("");
            }
        }
    };

    xhr.open("PUT", "https://www.sololo.cn/cndev/api/posts/" + id, true);
    xhr.setRequestHeader("Content-Type", "application/json");

    xhr.setRequestHeader("Authorization", "Bearer " + user.token);

    xhr.send(JSON.stringify(post));
}

function goBack() {
    window.history.back();
}

function copyUrl() {
    var sharingPathInput = document.getElementById("sharing_path");
    sharingPathInput.select();
    sharingPathInput.setSelectionRange(0, 99999); // For mobile devices
    var sharingUrl = document.getElementById("sharing_url_prefix").innerHTML + sharingPathInput.value;
    navigator.clipboard.writeText(sharingUrl).then(() => {
        alert("已复制: " + sharingUrl);
    }).catch(err => {
        console.error('复制失败: ', err);
    });
}

function generatePath() {
    var title = document.getElementById("title").value;

    if (title.length < 1) return;

    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                var res = "";
                eval("res = " + xhr.responseText);

                if (typeof res.error != "undefined") {
                    console.log(res);

                    return;
                }

                var translatedText = res[0].translations[0].text;
                var sharingPath = translatedText.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^\-/, "").replace(/\-$/, "");
                document.getElementById("sharing_path").value = sharingPath;
            } else {
                console.log(xhr.status);
                showToast("");
            }
        }
    };

    xhr.open("POST", "/_translate", true);

    xhr.setRequestHeader("Content-Type", "application/json; charset=UTF-8");

    var payload = [ { "Text": title } ];

    xhr.send(JSON.stringify(payload));
}

function loadStylesheet(url) {
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = url;
    link.onload = function() { console.log("CSS file " + url + " loaded"); };
    link.onerror = function() { console.log("Failed to load CSS file " + url); };
    document.head.appendChild(link);
}

window.addEventListener('load', function () {
    var postId = window.location.href.replace(/.*\//, "");

    var user_json = localStorage.getItem('user');
    var user = "";
    eval("user = " + user_json);

    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                var post = "";
                eval("post = " + xhr.responseText);

                onPost(user, post);
            } else {
                console.log(xhr.status);
                showToast("");
            }
        }
    };

    xhr.open("GET", "https://www.sololo.cn/cndev/api/posts/" + postId, true);

    xhr.setRequestHeader("Authorization", "Bearer " + user.token);

    xhr.send(null);
});