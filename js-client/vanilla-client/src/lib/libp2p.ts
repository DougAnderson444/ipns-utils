import { createLibp2p, Libp2p } from "libp2p"
import { webRTC, webRTCDirect } from "@libp2p/webrtc"
import { webTransport } from "@libp2p/webtransport"
import { gossipsub } from "@chainsafe/libp2p-gossipsub"
import { sha256 } from "multiformats/hashes/sha2"
import {
    CHAT_TOPIC,
    WEBRTC_BOOTSTRAP_NODE,
    CIRCUIT_RELAY_CODE,
} from "./constants"
import { noise } from "@chainsafe/libp2p-noise"
import { yamux } from "@chainsafe/libp2p-yamux"
import { kadDHT } from "@libp2p/kad-dht"
import { bootstrap } from "@libp2p/bootstrap"
import { multiaddr } from "@multiformats/multiaddr"
// @ts-ignore
import { circuitRelayTransport } from "libp2p/circuit-relay"

import type { Message, SignedMessage } from "@libp2p/interface-pubsub"
import type { Multiaddr } from "@multiformats/multiaddr"
import { copyToClipboard, setCircuit } from "../main"

export async function startLibp2p() {
    // localStorage.debug = "libp2p*,-*:trace" // if you wanted to exclude aything containing "gossipsub", you would add -gossipsub
    // localStorage.debug = "libp2p:connection-manager:dial-queue"

    const libp2p = await createLibp2p({
        dht: kadDHT({
            protocolPrefix: "/universal-connectivity",
            maxInboundStreams: 5000,
            maxOutboundStreams: 5000,
            clientMode: true,
        }),
        transports: [
            webRTC(), // for browser-to-browser
            webRTCDirect(), // for browser to server
            webTransport(),
            circuitRelayTransport({
                discoverRelays: 1,
            }),
        ],
        connectionEncryption: [noise()],
        streamMuxers: [yamux()],
        // peerDiscovery: [
        //     bootstrap({
        //         list: [
        //             WEBRTC_BOOTSTRAP_NODE,
        //             // WEBTRANSPORT_BOOTSTRAP_NODE,
        //         ],
        //         tagTTL: 31536000000, // 100 years in ms 100 * 60 * 60 * 24 * 365 * 1000 = 31536000000
        //     }),
        // ],
        pubsub: gossipsub({
            allowPublishToZeroPeers: true,
            msgIdFn: msgIdFnStrictNoSign,
            ignoreDuplicatePublishError: true,
            emitSelf: true,
        }),
        identify: {
            // these are set because we were seeing a lot of identify and identify push
            // stream limits being hit
            maxPushOutgoingStreams: 1000,
            maxPushIncomingStreams: 1000,
            maxInboundStreams: 1000,
            maxOutboundStreams: 1000,
        },
        autonat: {
            startupDelay: 60 * 60 * 24 * 1000,
        },
    })

    libp2p.pubsub.subscribe(CHAT_TOPIC)

    libp2p.peerStore.addEventListener(
        "change:multiaddrs",
        ({ detail: { peerId, multiaddrs } }) => {
            console.log(
                `changed multiaddrs: peer ${peerId.toString()} multiaddrs: ${multiaddrs}`
            )
            setWebRTCRelayAddress(multiaddrs, libp2p.peerId.toString())

            const connListEls = libp2p.getConnections().map((connection) => {
                return connection.remoteAddr.toString()
            })

            console.log("connections: ", connListEls)

            // findPeer
        }
    )

    return libp2p
}

export const setWebRTCRelayAddress = (maddrs: Multiaddr[], peerId: string) => {
    maddrs.forEach((maddr) => {
        if (maddr.protoCodes().includes(CIRCUIT_RELAY_CODE)) {
            const webRTCrelayAddress = multiaddr(
                maddr.toString() + "/webrtc/p2p/" + peerId
            )

            console.log(`Listening on '${webRTCrelayAddress.toString()}'`)

            setCircuit(webRTCrelayAddress.toString())
            copyToClipboard(document.querySelector<HTMLButtonElement>("#copy")!)
        }
    })
}

// message IDs are used to dedup inbound messages
// every agent in network should use the same message id function
// messages could be perceived as duplicate if this isnt added (as opposed to rust peer which has unique message ids)
export async function msgIdFnStrictNoSign(msg: Message): Promise<Uint8Array> {
    var enc = new TextEncoder()

    const signedMessage = msg as SignedMessage
    const encodedSeqNum = enc.encode(signedMessage.sequenceNumber.toString())
    return await sha256.encode(encodedSeqNum)
}

export const connectToMultiaddr =
    (libp2p: Libp2p) => async (multiaddr: Multiaddr) => {
        console.log(`dialing: ${multiaddr.toString()}`)
        try {
            const conn = await libp2p.dial(multiaddr)
            // or dialProtocol
            // const conn = await libp2p.dialProtocol(multiaddr, [
            //     "/floodsub/1.0.0",
            //     "/ipfs/id/1.0.0",
            //     "/ipfs/id/push/1.0.0",
            //     "/ipfs/ping/1.0.0",
            //     "/libp2p/autonat/1.0.0",
            //     "/libp2p/fetch/0.0.1",
            //     "/meshsub/1.1.0",
            //     "/universal-connectivity/lan/kad/1.0.0",
            //     "/universal-connectivity/1.0.0",
            // ])
            return conn
        } catch (e) {
            console.error(e)
            throw e
        }
    }

export const publishMessage = (libp2p: Libp2p) => async (message: string) => {
    try {
        const res = await libp2p.pubsub.publish(
            CHAT_TOPIC,
            new TextEncoder().encode(message)
        )

        console.log(
            "sent message to: ",
            res.recipients.map((peerId) => peerId.toString())
        )
    } catch (e) {
        console.error(e)
        throw e
    }
}
