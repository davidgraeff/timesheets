import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	css: {
		preprocessorOptions: {
			scss: {
				additionalData: '@use "src/variables.scss" as *;'
			}
		}
	},
	server: {
		proxy: {
			'/api': {
				target: 'http://0.0.0.0:8080',
				changeOrigin: true,
				// rewrite: (path) => path.replace(/^\/api/, ''),
			},
		}
	}
});
