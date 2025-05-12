import {
    defineConfig,
    presetAttributify,
    presetIcons,
    presetTypography,
    presetUno,
    presetWebFonts,
    transformerDirectives,
    transformerVariantGroup,
} from 'unocss';

export default defineConfig({
    shortcuts: [
        [
            'btn',
            'px-4 py-1 rounded inline-block cursor-pointer disabled:cursor-default disabled:bg-gray-600 disabled:opacity-50',
        ],
        [
            'icon-btn',
            'inline-block cursor-pointer select-none opacity-75 transition duration-200 ease-in-out hover:opacity-100 hover:text-teal-600',
        ],
        ['shadow-box', 'shadow bg-white p-5 rd-3'],
        ['dialog-box', 'w-100 bg-gray-5 rd-5 p-5 pr-10 relative text-white line-height-normal shadow-gray-4 shadow']
    ],
    presets: [
        presetUno(),
        presetAttributify(),
        presetIcons({
            scale: 1.2,
        }),
        presetTypography(),
        presetWebFonts({
            fonts: {
                sans: 'DM Sans',
                serif: 'DM Serif Display',
                mono: 'DM Mono',
            },
        }),
    ],
    transformers: [transformerDirectives(), transformerVariantGroup()],
    theme: {
        colors: {},
    },
});
