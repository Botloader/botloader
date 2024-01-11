import { useEffect, useState } from "react";
import untar from "js-untar";
import useMonacoFixed from "./useMonacoFixed";

export function useBotloaderMonaco(extraSources?: { name: string, content: string }[]) {
    const monaco = useMonacoFixed();
    const [init, setInit] = useState(false);
    const [typings, setTypings] = useState<File[] | undefined | null>(undefined);

    useEffect(() => {
        if (!monaco || !typings) {
            return;
        }

        monaco.languages.typescript.typescriptDefaults.setExtraLibs(
            [
                ...typings.filter(v => v.type === "0")
                    .map(v => {
                        return {
                            content: v.readAsString(),
                            filePath: "file:///" + v.name,
                        }
                    }),
                ...(extraSources?.map(v => {
                    return {
                        content: v.content,
                        filePath: "file:///" + v.name + ".ts"
                    }
                }) ?? [])
            ]
        )

        monaco.languages.typescript.typescriptDefaults.setInlayHintsOptions({
            includeInlayFunctionLikeReturnTypeHints: true,
            includeInlayParameterNameHints: "all",
            includeInlayVariableTypeHints: true,
            includeInlayFunctionParameterTypeHints: true,
            includeInlayPropertyDeclarationTypeHints: true,
            includeInlayEnumMemberValueHints: true
        })

        monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
            moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
            module: monaco.languages.typescript.ModuleKind.ESNext,
            lib: [
                "es2021",
            ],
            allowNonTsExtensions: true,
            noImplicitAny: true,
            removeComments: true,
            preserveConstEnums: true,
            sourceMap: false,
            target: monaco.languages.typescript.ScriptTarget.ESNext,
            alwaysStrict: true,
            strict: true,
            strictNullChecks: true,
            paths: {
                "botloader": ["file:///typings/index.d.ts"]
            }
        })

        setInit(true);
    }, [monaco, typings, extraSources]);

    useEffect(() => {
        async function loadTypings() {
            let files = await downloadTypeDecls();
            setTypings(files);
        }

        loadTypings();
    }, []);


    return init;
}

async function downloadTypeDecls(): Promise<File[]> {
    let resp = await fetch("/typings.tar");
    let res = await untar(await resp.arrayBuffer());
    return res
}

interface File {
    name: string,
    mode: string,
    type: string,

    readAsString(): string,
    readAsJSON(): unknown,
}