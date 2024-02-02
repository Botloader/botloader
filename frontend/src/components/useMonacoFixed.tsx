// useMonaco in monaco-editor/react is broken in strict mode, so i removed the troublesome portion in my copy here

import { useEffect, useState } from 'react';
import loader from '@monaco-editor/loader';

function useMonacoFixed() {
    const [monaco, setMonaco] = useState(loader.__getMonacoInstance());

    useEffect(() => {
        let cancelable: ReturnType<typeof loader.init>;

        if (!monaco) {
            cancelable = loader.init();

            cancelable.then((monaco) => {
                setMonaco(monaco);
            });
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    return monaco;
}

export default useMonacoFixed;
