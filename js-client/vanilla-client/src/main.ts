import "./style.css"
import "./index.css"
import {
    publishMessage,
    startLibp2p,
    connectToMultiaddr,
} from "./lib/libp2p.ts"
import { multiaddr } from "@multiformats/multiaddr"

let myPeerId: string
let anchor = document.querySelector("#anchor")

// check if in browser, then startLibp2p()
if (typeof window !== "undefined") {
    // anonymous async function
    ;(async () => {
        const libp2p = await startLibp2p()

        // @ts-ignore
        window.libp2p = libp2p

        // set #my-peerid to libp2p.peerId.toB58String()
        myPeerId = libp2p.peerId.toString()
        document.querySelector<HTMLDivElement>("#my-peerid")!.textContent =
            myPeerId

        libp2p.addEventListener("peer:connect", peerConnected)
        // @ts-ignore
        libp2p.pubsub.addEventListener("message", handleMessage)

        autoScroller()
        enterListener()
    })()
}

setupConnector(document.querySelector<HTMLButtonElement>("#connect")!)
setupPublisher(document.querySelector<HTMLButtonElement>("#publish")!)

// set id="circuit" value fn
export function setCircuit(value: string) {
    document.querySelector<HTMLInputElement>("#circuit")!.value = value
}

// copy to clipboard when id="copy" button is clicked
export function copyToClipboard(element: HTMLButtonElement) {
    element.addEventListener("click", async () => {
        const circuit = document.querySelector<HTMLInputElement>("#circuit")!
        circuit.select()
        circuit.setSelectionRange(0, 99999)
        // without using         document.execCommand("copy"), use alternate method
        // https://stackoverflow.com/questions/400212/how-do-i-copy-to-the-clipboard-in-javascript
        await navigator.clipboard.writeText(circuit.value)
    })
}

// peerConnected(event) callback sets list of peers in browser
function peerConnected(event: CustomEvent) {
    const { remoteAddr } = event.detail

    console.log("peerConnected: ", event.detail.remoteAddr.toString())

    //  add remoteAddr into multiaddr only if unique
    const multiaddr = remoteAddr.toString()

    // check if any of the html element <li> nodes contain the text string ${multiaddr}
    const multiaddrElements =
        document.querySelectorAll<HTMLLIElement>("#multiaddrs li")

    let exists = false
    multiaddrElements.forEach((element) => {
        if (element.textContent == multiaddr) {
            exists = true
        }
    })
    if (exists) return

    const multiaddrElement = document.createElement("li")

    // style the li with Tailwindcss
    // remove li decoration, fit text size to width, and add padding
    multiaddrElement.classList.add(
        "text-neutral-700",
        "text-sm",
        "truncate",
        "px-2",
        "py-1",
        "list-none"
    )
    multiaddrElement.textContent = multiaddr
    document
        .querySelector<HTMLUListElement>("#multiaddrs")!
        .appendChild(multiaddrElement)
}

// handle messages using libp2p.pubsub.addEventListener('message', messageCB)
function handleMessage(evt: CustomEvent) {
    console.log("handleMessage", evt.detail)

    const { topic, from, data } = evt.detail
    const msg = new TextDecoder().decode(data)
    console.log(`${topic}: ${msg}`)

    // Append signed messages, otherwise discard
    if (evt.detail.type === "signed") {
        // append to existing inner #messages div
        const messages = document.querySelector<HTMLDivElement>("#messages")!
        const message = document.createElement("div")
        const peerId = from.toString().slice(-4)

        // if from == myPeerId, then align right. otherwise, align left using Tailwindcss classes
        // get last 4 of peerId from `from` :
        message.classList.add("flex", "items-center", "mb-2")
        let who = "them"
        if (from.toString() == myPeerId) {
            who = "me"
            message.classList.add("flex-row-reverse")
        }
        const messagePeerId = document.createElement("div")
        messagePeerId.classList.add(
            "flex",
            "flex-col",
            "items-center",
            "justify-center",
            "w-12",
            "h-12",
            "m-2",
            "text-neutral-700",
            who == "them" ? "bg-blue-100" : "bg-green-100",
            "rounded-full"
        )
        messagePeerId.textContent = peerId
        message.appendChild(messagePeerId)
        const messageText = document.createElement("div")
        messageText.classList.add(
            "flex",
            "flex-col",
            "items-start",
            "justify-center",
            "w-2/3",
            "h-12",
            "px-4",
            "py-2",
            "text-neutral-700",
            who == "them" ? "bg-blue-100" : "bg-green-100",
            "rounded-lg"
        )
        messageText.textContent = msg
        message.appendChild(messageText)
        messages.insertBefore(message, anchor)

        // message.textContent = `${from.toString()}: ${msg}`
        // messages.appendChild(message)
    }
}

// setupPublisher(element) callback
function setupPublisher(element: HTMLButtonElement) {
    element.addEventListener("click", async () => {
        // get message from input element
        const message =
            document.querySelector<HTMLInputElement>("#message")!.value

        // publish message to topic
        await publishMessage(
            // @ts-ignore
            window.libp2p
        )(message)
    })
}

function enterListener() {
    // Enter input listener on #message, click #publish button
    document
        .querySelector<HTMLInputElement>("#message")!
        .addEventListener("keyup", (event) => {
            if (event.key === "Enter") {
                document.querySelector<HTMLButtonElement>("#publish")!.click()
            }
        })
}
function autoScroller() {
    const scrollingElement = document.getElementById("messages")

    const config = { childList: true }

    const callback = function (mutationsList: any) {
        for (let mutation of mutationsList) {
            if (scrollingElement && mutation.type === "childList") {
                scrollingElement.scrollTo(0, scrollingElement.scrollHeight)
            }
        }
    }

    const observer = new MutationObserver(callback)
    // @ts-ignore
    observer.observe(scrollingElement, config)
}
export function setupConnector(element: HTMLButtonElement) {
    // get from multiaddr input element

    element.addEventListener("click", async () => {
        let maddr =
            document.querySelector<HTMLInputElement>("#multiaddr")!.value

        const connection = await connectToMultiaddr(
            // @ts-ignore
            window.libp2p
        )(multiaddr(maddr))
        console.log("connection: ", connection)
    })
}
