crate_name := `echo ${PWD##*/}`
index_file := "<html lang='en'>

<head>
    <meta charset='utf-8'>
    <title>" + crate_name + "</title>
    <style>
        html,
        body,
        canvas {
            margin: 0px;
            padding: 0px;
            width: 100%;
            height: 100%;
            overflow: hidden;
            position: absolute;
            background: black;
            z-index: 0;
        }
    </style>
</head>

<body>
    <canvas id='" + crate_name + "' tabindex='1'></canvas>
    <script src='./mq_js_bundle.js'></script>
    <script>load('" + crate_name + ".wasm');</script> 
</body>

</html>"

build-web: 
    cargo build --profile=web-release --target wasm32-unknown-unknown
    wget -O ./target/wasm32-unknown-unknown/web-release/mq_js_bundle.js https://raw.githubusercontent.com/not-fl3/macroquad/master/js/mq_js_bundle.js 
    @sed -i -e 's/#glcanvas/#{{crate_name}}/g' ./target/wasm32-unknown-unknown/web-release/mq_js_bundle.js
    @echo "{{index_file}}" > ./target/wasm32-unknown-unknown/web-release/index.html
    wasm-opt -Oz -o ./target/wasm32-unknown-unknown/web-release/{{crate_name}}.wasm ./target/wasm32-unknown-unknown/web-release/{{crate_name}}.wasm 
    wasm-snip ./target/wasm32-unknown-unknown/web-release/{{crate_name}}.wasm  -o ./target/wasm32-unknown-unknown/web-release/{{crate_name}}.wasm 

