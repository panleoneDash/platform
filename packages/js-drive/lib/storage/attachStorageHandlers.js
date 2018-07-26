const STHeadersReader = require('../blockchain/reader/STHeadersReader');
const ArrayBlockIterator = require('../blockchain/iterator/ArrayBlockIterator');
const StateTransitionHeaderIterator = require('../blockchain/iterator/StateTransitionHeaderIterator');
const rejectAfter = require('../util/rejectAfter');
const InvalidPacketCidError = require('./InvalidPacketCidError');

const PIN_REJECTION_TIMEOUT = 1000 * 60 * 3;

/**
 * Add State Transition Packet from blockchain when new ST header will appear.
 * Remove State Transition Packet from blockchain when wrong sequence.
 * Remove all State Transition Packets from blockchain when reset.
 *
 * @param {STHeadersReader} stHeadersReader
 * @param {IpfsAPI} ipfsAPI
 * @param {unpinAllIpfsPackets} unpinAllIpfsPackets
 */
function attachStorageHandlers(stHeadersReader, ipfsAPI, unpinAllIpfsPackets) {
  const { stHeaderIterator: { rpcClient } } = stHeadersReader;

  stHeadersReader.on(STHeadersReader.EVENTS.HEADER, async ({ header }) => {
    const pinPromise = ipfsAPI.pin.add(header.getPacketCID(), { recursive: true });
    const error = new InvalidPacketCidError();

    await rejectAfter(pinPromise, error, PIN_REJECTION_TIMEOUT);
  });

  stHeadersReader.on(STHeadersReader.EVENTS.STALE_BLOCK, async (block) => {
    const blockIterator = new ArrayBlockIterator([block]);
    const stHeadersIterator = new StateTransitionHeaderIterator(blockIterator, rpcClient);

    let done;
    let header;

    // eslint-disable-next-line no-cond-assign
    while ({ done, value: header } = await stHeadersIterator.next()) {
      if (done) {
        break;
      }

      await ipfsAPI.pin.rm(header.getPacketCID(), { recursive: true });
    }
  });

  stHeadersReader.on(STHeadersReader.EVENTS.RESET, async () => {
    await unpinAllIpfsPackets();
  });
}

Object.assign(attachStorageHandlers, {
  PIN_REJECTION_TIMEOUT,
});

module.exports = attachStorageHandlers;