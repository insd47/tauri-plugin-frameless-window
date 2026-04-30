import typescript from '@rollup/plugin-typescript';
import dts from 'rollup-plugin-dts';

const external = ['@tauri-apps/api/core'];

export default [
  {
    input: 'guest-js/index.ts',
    output: {
      dir: 'dist-js',
      format: 'es',
      preserveModules: true,
      preserveModulesRoot: 'guest-js',
    },
    plugins: [
      typescript({
        tsconfig: './tsconfig.json',
        declaration: true,
        declarationDir: './dist-js',
      }),
    ],
    external,
  },
  {
    input: 'guest-js/index.ts',
    output: {
      file: 'dist-js/index.cjs',
      format: 'cjs',
    },
    plugins: [
      typescript({
        tsconfig: './tsconfig.json',
        declaration: false,
        declarationDir: null,
      }),
    ],
    external,
  },
  {
    input: 'dist-js/index.d.ts',
    output: {
      file: 'dist-js/index.d.ts',
      format: 'es',
    },
    plugins: [dts()],
    external,
  },
];
