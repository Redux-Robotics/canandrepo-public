(function(f){f(f);})(function(f) {
    if (typeof pathtoroot !== "undefined") {
        var script = document.createElement("script");
        script.src = pathtoroot + "highlight.min.js";
        script.type = "text/javascript";
        script.onload = function() {
            var script = document.createElement("script");
            script.src = pathtoroot + "java.min.js";
            script.type = "text/javascript";
            script.onload = function() {
                window.hljs.configure({cssSelector: "pre", languages: ["java"], noHighlightRe: /^no-code$/i});
                window.hljs.highlightAll();
            };
            document.head.appendChild(script);
        };
        document.head.appendChild(script);

    } else {
        setTimeout(f, 100);
    }
});