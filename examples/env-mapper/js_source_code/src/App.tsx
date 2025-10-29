import {useState} from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import './App.css'

function App() {
    const [_, setCount] = useState(0);
    const apiUrl = import.meta.env.VITE_API_URL;
    const title = import.meta.env.VITE_TITLE_PAGE;

    return (
        <>
            <div>
                <a href="https://vite.dev" target="_blank">
                    <img src={viteLogo} className="logo" alt="Vite logo"/>
                </a>
                <a href="https://react.dev" target="_blank">
                    <img src={reactLogo} className="logo react" alt="React logo"/>
                </a>
            </div>
            <h1>Vite + React</h1>
            <div className="card">
                <button onClick={() => setCount((count) => count + 1)}>
                    Var: {apiUrl}
                </button>
            </div>
            <p className="read-the-docs">
                Click on the Vite and React logos to learn more {title}
            </p>
        </>
    )
}

export default App
