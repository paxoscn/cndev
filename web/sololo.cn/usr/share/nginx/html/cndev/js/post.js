import mermaid from 'https://www.sololo.cn/cndev/js/mermaid/mermaid.esm.min.js';
    
mermaid.initialize({ startOnLoad: false });

var mermaid_svg_index = 0;

const mermaid_ext = {
    name: 'mermaid_ext',
    level: 'block',
    start(src) { return src.match(/```mermaid/)?.index; },
    tokenizer(src, tokens) {
        const rule = /^(?:```mermaid\n([^`]+)```)+/;
        const match = rule.exec(src);
        if (match) {
            const token = {
                type: 'mermaid_ext',
                raw: match[0],
                text: match[1],
                tokens: []
            };
            this.lexer.inline(token.text, token.tokens);
            return token;
        }
    },
    renderer(token) {
        var svgEl = document.createElement('div');
        var svgElId = "mermaid_svg" + mermaid_svg_index++;
        svgEl.setAttribute("id", svgElId);
        svgEl.style.display = "none";
        document.body.appendChild(svgEl);
        mermaid.render(svgElId, token.text)
            .then(svgObj => {
                setTimeout(() => {
                    document.getElementById(svgElId + "_to_render").innerHTML = svgObj.svg;
                }, 250);
            });
        return "<div id=\"" + svgElId + "_to_render\"></div>"
    }
};

window.onInit = function(user) {
    document.getElementById("content_main").style.display = "block";

    author_nick = window.location.href.replace(/\/$/, "").replace(/.*\/([^\/]+)\/.*/, "$1");

    tryApplyAvatar(author_id);

    document.querySelector(".author_nick").setAttribute("href", "/" + (author_nick.length > 0 ? author_nick : author_id));
    document.querySelector(".author_nick").innerHTML = author_nick;
    document.querySelector(".author_nick").addEventListener('click', function (e) {
        window.location.href = "/" + author_nick;
    });

    if (user != null && user.id == author_id) {
        document.querySelector(".edit").style.display = "block";
    }

    marked.use({ extensions: [ mermaid_ext ] });

    var raw_text = new DOMParser().parseFromString(document.getElementById("post_text").innerHTML, "text/html").documentElement.textContent;
    document.getElementById("post_rendered_text").innerHTML = marked.parse(raw_text);
};