import { defineStore } from "pinia";
import { ref } from "vue";
import { useMessengerStore } from "./messengerStore";

export const useCallStore = defineStore("call", () => {
	const messenger = useMessengerStore();

	// WebRTC State
	const peerConnection = ref<RTCPeerConnection | null>(null);
	const localStream = ref<MediaStream | null>(null);
	const remoteStream = ref<MediaStream | null>(null);

	// Call State
	const activeCallChatId = ref<string | null>(null);
	const isMuted = ref(false);
	const callState = ref<"idle" | "calling" | "ringing" | "connected">("idle");
	// If true, we are the one initiating the call
	const isCaller = ref(false);
	const queuedIceCandidates = ref<RTCIceCandidateInit[]>([]);
	const incomingOfferSdp = ref<string | null>(null);

	// STUN Servers for ICE candidates
	const rtcConfig: RTCConfiguration = {
		iceServers: [
			{ urls: "stun:stun.l.google.com:19302" },
			{ urls: "stun:stun1.l.google.com:19302" },
		],
	};

	async function getLocalMedia() {
		try {
			localStream.value = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
		} catch (err) {
			console.error("Failed to get local media", err);
			throw err;
		}
	}

	function createPeerConnection(chatUuid: string) {
		if (peerConnection.value) {
			peerConnection.value.close();
		}

		peerConnection.value = new RTCPeerConnection(rtcConfig);
		queuedIceCandidates.value = [];

		// Add local tracks to PC
		if (localStream.value) {
			localStream.value.getTracks().forEach((track) => {
				peerConnection.value?.addTrack(track, localStream.value!);
			});
		}

		// Handle remote tracks
		peerConnection.value.ontrack = (event) => {
			console.debug("[WebRTC] Received remote track", event.streams);
			if (event.streams && event.streams[0]) {
				remoteStream.value = event.streams[0];
			} else {
				if (!remoteStream.value) {
					remoteStream.value = new MediaStream();
				}
				remoteStream.value.addTrack(event.track);
			}
		};

		// Handle ICE candidates
		peerConnection.value.onicecandidate = (event) => {
			if (event.candidate && messenger.wsStatus === "OPEN") {
				messenger.sendRawWsMessage({
					action: "ice_candidate",
					payload: {
						chat_uuid: chatUuid,
						candidate: event.candidate.candidate,
						sdp_mid: event.candidate.sdpMid,
						sdp_m_line_index: event.candidate.sdpMLineIndex,
					},
				});
			}
		};

		peerConnection.value.onconnectionstatechange = () => {
			console.debug("[WebRTC] Connection state:", peerConnection.value?.connectionState);
			if (peerConnection.value?.connectionState === "connected") {
				callState.value = "connected";
			} else if (peerConnection.value?.connectionState === "closed") {
				endCall();
			} else if (peerConnection.value?.connectionState === "failed") {
				console.warn("[WebRTC] Connection failed. Ending call.");
				endCall();
			}
			// Let 'disconnected' hang for a bit as it might recover, but log it
			else if (peerConnection.value?.connectionState === "disconnected") {
				console.warn("[WebRTC] Connection disconnected, waiting for possible reconnect...");
			}
		};
	}

	// ─── Actions ─────────────────────────────────────────────────────────

	async function startCall(chatUuid: string) {
		if (callState.value !== "idle") return;
		try {
			await getLocalMedia();
			activeCallChatId.value = chatUuid;
			isCaller.value = true;
			callState.value = "calling";

			createPeerConnection(chatUuid);

			const offer = await peerConnection.value!.createOffer();
			await peerConnection.value!.setLocalDescription(offer);

			messenger.sendRawWsMessage({
				action: "call_offer",
				payload: {
					chat_uuid: chatUuid,
					sdp: JSON.stringify(offer),
				},
			});
		} catch (err) {
			console.error("Failed to start call", err);
			endCall();
		}
	}

	async function receiveOffer(chatUuid: string, _callerUuid: string, sdpStr: string) {
		if (callState.value !== "idle") {
			// Busy
			messenger.sendRawWsMessage({
				action: "call_reject",
				payload: { chat_uuid: chatUuid },
			});
			return;
		}

		activeCallChatId.value = chatUuid;
		isCaller.value = false;
		callState.value = "ringing";

		// We store the offer in state temporarily to apply it on answer
		// but since we want to ring first, we don't setRemoteDescription yet.
		// So we save the sdp string in a local ref to apply when answered
		incomingOfferSdp.value = sdpStr;
	}

	async function answerCall() {
		if (callState.value !== "ringing" || !activeCallChatId.value || !incomingOfferSdp.value) return;

		try {
			await getLocalMedia();
			createPeerConnection(activeCallChatId.value);

			const offer: RTCSessionDescriptionInit = JSON.parse(incomingOfferSdp.value);
			await peerConnection.value!.setRemoteDescription(offer);
			flushIceCandidates();

			const answer = await peerConnection.value!.createAnswer();
			await peerConnection.value!.setLocalDescription(answer);

			messenger.sendRawWsMessage({
				action: "call_answer",
				payload: {
					chat_uuid: activeCallChatId.value,
					sdp: JSON.stringify(answer),
				},
			});

			callState.value = "connected";
		} catch (err) {
			console.error("Failed to answer call", err);
			endCall();
		}
	}

	async function receiveAnswer(chatUuid: string, _responderUuid: string, sdpStr: string) {
		if (callState.value === "calling" && activeCallChatId.value === chatUuid) {
			try {
				const answer: RTCSessionDescriptionInit = JSON.parse(sdpStr);
				await peerConnection.value!.setRemoteDescription(answer);
				flushIceCandidates();
				callState.value = "connected";
			} catch (err) {
				console.error("Failed to set remote description from answer", err);
			}
		}
	}

	async function receiveIceCandidate(chatUuid: string, candidate: string, sdpMid: string | null, sdpMLineIndex: number | null) {
		if (activeCallChatId.value === chatUuid && peerConnection.value) {
			const iceCandidate = { candidate, sdpMid, sdpMLineIndex };
			if (peerConnection.value.remoteDescription) {
				try {
					await peerConnection.value.addIceCandidate(iceCandidate);
				} catch (err) {
					console.error("Failed to add ICE candidate", err);
				}
			} else {
				queuedIceCandidates.value.push(iceCandidate);
			}
		}
	}

	function flushIceCandidates() {
		for (const candidate of queuedIceCandidates.value) {
			if (peerConnection.value?.remoteDescription) {
				peerConnection.value.addIceCandidate(candidate).catch((err) => {
					console.error("Failed to add queued ICE candidate", err);
				});
			}
		}
		queuedIceCandidates.value = [];
	}

	function rejectCall() {
		if (activeCallChatId.value) {
			messenger.sendRawWsMessage({
				action: "call_reject",
				payload: { chat_uuid: activeCallChatId.value },
			});
		}
		endCall();
	}

	function endCall() {
		if (activeCallChatId.value && callState.value !== "idle") {
			messenger.sendRawWsMessage({
				action: "call_end",
				payload: { chat_uuid: activeCallChatId.value },
			});
		}

		_cleanup();
	}

	function handleRemoteCallEnd(chatUuid: string) {
		if (activeCallChatId.value === chatUuid) {
			_cleanup();
		}
	}

	function handleRemoteCallReject(chatUuid: string) {
		if (activeCallChatId.value === chatUuid) {
			_cleanup();
		}
	}

	function toggleMute() {
		if (localStream.value) {
			localStream.value.getAudioTracks().forEach((track) => {
				track.enabled = !track.enabled;
			});
			isMuted.value = !localStream.value.getAudioTracks()[0]?.enabled;
		}
	}

	function _cleanup() {
		if (peerConnection.value) {
			peerConnection.value.close();
			peerConnection.value = null;
		}
		if (localStream.value) {
			localStream.value.getTracks().forEach((t) => t.stop());
			localStream.value = null;
		}
		remoteStream.value = null;
		activeCallChatId.value = null;
		callState.value = "idle";
		incomingOfferSdp.value = null;
		isCaller.value = false;
		isMuted.value = false;
		queuedIceCandidates.value = [];
	}

	return {
		localStream,
		remoteStream,
		activeCallChatId,
		callState,
		isMuted,
		isCaller,
		startCall,
		receiveOffer,
		answerCall,
		receiveAnswer,
		receiveIceCandidate,
		rejectCall,
		endCall,
		handleRemoteCallEnd,
		handleRemoteCallReject,
		toggleMute,
	};
});
