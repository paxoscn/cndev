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
    document.querySelector(".category[value=\"" + post.category + "\"]").checked = true;
    document.getElementById("the_abstract").value = post.the_abstract;
    document.getElementById("text").value = post.text;
    if (post.references.length > 0) {
        var referenceLines = post.references.split("\n");
        referenceLines.forEach((referenceLine) => {
            var url = referenceLine.substring(0, referenceLine.indexOf(" "));
            var title = referenceLine.substring(referenceLine.indexOf(" ") + 1);
            var referenceDiv = document.querySelector(".reference_templates > .reference").cloneNode(true);
            referenceDiv.querySelector(".reference_title").value = title;
            referenceDiv.querySelector(".reference_url").value = url;
            referenceDiv.querySelector(".button_reference_removing").addEventListener("click", removeReference);
            document.querySelector(".references").appendChild(referenceDiv);
        });
    }

    document.querySelectorAll(".category").forEach((category) => {
        category.addEventListener("change", (e) => {
            document.getElementById("p_abstract").style.display = e.target.getAttribute("value") == "1" ? "" : "none";
        });
    });

    document.getElementById("button_reference_adding").addEventListener("click", (e) => {
        var referenceDiv = document.querySelector(".reference_templates > .reference").cloneNode(true);
        referenceDiv.querySelector(".button_reference_removing").addEventListener("click", removeReference);
        document.querySelector(".references").appendChild(referenceDiv);
    });

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

    simplemde.codemirror.on("paste", onPaste);
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
    post.category = parseInt(document.querySelector(".category:checked").value);
    post.the_abstract = document.getElementById("the_abstract").value;
    post.text = simplemde.value();
    post.references = "";
    var referenceDivs = document.querySelectorAll(".reference");
    referenceDivs.forEach((referenceDiv) => {
        var url = referenceDiv.querySelector(".reference_url").value.trim();
        var title = referenceDiv.querySelector(".reference_title").value.trim();
        if (url.length > 0 || title.length > 0) {
            var referenceLine = url + " " + title;
            post.references += (post.references.length > 0 ? "\n" : "") + referenceLine;
        }
    });

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

function onPaste(ideEvent, docEvent) {
    // We need to check if event.clipboardData is supported (Chrome & IE)
    if (docEvent.clipboardData && docEvent.clipboardData.items) {
        // Get the items from the clipboard
        var items = docEvent.clipboardData.items;
  
        // Loop through all items, looking for any kind of image
        for (var i = 0; i < items.length; i++) {
            if (items[i].type.indexOf('image') !== -1) {
                // We need to represent the image as a file
                var blob = items[i].getAsFile();

                uploadImage(ideEvent.doc, blob);
  
                // Prevent the image (or URL) from being pasted into the contenteditable element
                docEvent.preventDefault();
            }
        }
    }
}

function uploadImage(cmDoc, blob) {
    var user_json = localStorage.getItem('user');
    var user = "";
    eval("user = " + user_json);
    
    var id = document.getElementById("id").value;

    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                var res = "";
                eval("res = " + xhr.responseText);

                cmDoc.replaceRange("![" + blob.name + "](https://www.sololo.cn/cndev/_post_images/" + res.uploaded_file_name + ")", cmDoc.getCursor("start"), cmDoc.getCursor("end"));
            } else if (xhr.status === 401) {
                localStorage.removeItem('user')

                window.location.reload();
            } else {
                showToast("");
            }
        }
    };

    xhr.open("PUT", "https://www.sololo.cn/cndev/api/posts/" + id + "/images", true);

    xhr.setRequestHeader("Authorization", "Bearer " + user.token);

    const formData = new FormData();
    formData.append("image", blob);

    xhr.send(formData);
}

function removeReference(e) {
    var referenceDiv = e.target.parentNode.parentNode;
    var title = referenceDiv.querySelector(".reference_title").value;
    var url = referenceDiv.querySelector(".reference_url").value;
    if ((title.length < 1 && url.length < 1) || confirm("确定要移除参考资料 " + (title.length > 0 ? title : "") + (url.length > 0 ? (" (" + url + ")") : "") + " 吗?")) {
        document.querySelector(".references").removeChild(referenceDiv);
    }
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