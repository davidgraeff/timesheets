import preprocess from 'svelte-preprocess';
import adapter from '@sveltejs/adapter-static';
import {vitePreprocess} from '@sveltejs/kit/vite';

/** @type {import('@sveltejs/kit').Config} */
const config = {
    // Consult https://kit.svelte.dev/docs/integrations#preprocessors
    // for more information about preprocessors
    preprocess: [
        vitePreprocess(),
        preprocess({
            scss: {
                prependData: '@use "src/variables.scss" as *;'
            }
        })
    ],

    kit: {
        paths: {
            base: "",
            relative: true
        },
        adapter: adapter({
            pages: 'build',
            assets: 'build',
            fallback: null,
            precompress: false,
            strict: true
        }),
        prerender: {
            handleHttpError: ({status, path, referrer, referenceType}) => {
                if (path.startsWith("/camera")) return;
                if (path.startsWith("/api")) return;
                console.warn(
                    `${status} ${path}${referrer ? ` (${referenceType} from ${referrer})` : ""}`
                );
            }
        }
    }
};

export default config;
