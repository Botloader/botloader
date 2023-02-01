import { Gist } from "../components/Gist";
import { Panel } from "../components/Panel";
import "./SamplesPage.css"

interface ISample {
    name: string,
    description: string,
    gist: string
}

const SAMPLES: ISample[] = [{
    name: "Levels",
    description: "A very simple leveling system with a leaderboard, you can adjust the `getLevel` function to change the level scaling",
    gist: "bbfca2054426fecb0d73dfe2ae601f79",
}, {
    name: "Role menu",
    description: "A simple select dropdown of roles, you have to edit this script to give it your server's roles and such.",
    gist: "a7541ae1f6530435dd70c27c53b45b23",
}, {
    name: "Giveaway",
    description: "A relatively complex script providing giveaways, creates a `giveaway create` command.",
    gist: "9e7945f8e0669d0707e85ecd97ef2dd3",
}];

export function SamplesPage() {
    return <>{SAMPLES.map(v => <Sample key={v.gist} entry={v}></Sample>)}</>
}

function Sample(props: { entry: ISample }) {
    return <Panel className="sample-panel">
        <h3>{props.entry.name}</h3>
        <p>{props.entry.description}</p>
        <Gist id={props.entry.gist} ></Gist>
    </Panel>
}