crate_name := `basename "$PWD" | tr '-' '_'`
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
    <script type='module' src='./mq_js_bundle.js'></script>
    <script type='module'>
        import { load } from './mq_js_bundle.js';

        load('" + crate_name + ".wasm');
    </script> 
</body>

</html>"

# Build the current project for the web along with template files
build-web: 
    @# build the program
    cargo build --profile=web-release --target wasm32-unknown-unknown

    @echo "{{index_file}}" > ./target/wasm32-unknown-unknown/web-release/index.html
    @cp ./assets/mq_js_bundle/mq_js_bundle.js ./target/wasm32-unknown-unknown/web-release/mq_js_bundle.js
    @sed -i -e 's/#CRATE_NAME/#{{crate_name}}/g' ./target/wasm32-unknown-unknown/web-release/mq_js_bundle.js
    
    @# run wasm binary optimization
    wasm-opt -Oz -o ./target/wasm32-unknown-unknown/web-release/{{crate_name}}.wasm ./target/wasm32-unknown-unknown/web-release/{{crate_name}}.wasm --enable-simd --enable-nontrapping-float-to-int --enable-bulk-memory-opt
    wasm-snip ./target/wasm32-unknown-unknown/web-release/{{crate_name}}.wasm  -o ./target/wasm32-unknown-unknown/web-release/{{crate_name}}.wasm 

