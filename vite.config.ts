import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	ssr: {
		noExternal: process.env.NODE_ENV === 'production' ? ['@carbon/charts', 'carbon-components'] : []
	},
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
				target: 'http://127.0.0.1:8099',
				changeOrigin: true,
				// rewrite: (path) => path.replace(/^\/api/, ''),
			},
		}
	}
});
