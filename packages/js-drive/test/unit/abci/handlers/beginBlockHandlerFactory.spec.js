const {
  abci: {
    ResponseBeginBlock,
  },
} = require('abci/types');

const beginBlockHandlerFactory = require('../../../../lib/abci/handlers/beginBlockHandlerFactory');

const BlockchainState = require('../../../../lib/blockchainState/BlockchainState');
const BlockExecutionDBTransactionsMock = require('../../../../lib/test/mock/BlockExecutionDBTransactionsMock');
const BlockExecutionStateMock = require('../../../../lib/test/mock/BlockExecutionStateMock');

describe('beginBlockHandlerFactory', () => {
  let beginBlockHandler;
  let request;
  let blockchainState;
  let blockHeight;
  let blockExecutionDBTransactionsMock;
  let blockExecutionStateMock;

  beforeEach(function beforeEach() {
    blockchainState = new BlockchainState();

    blockExecutionDBTransactionsMock = new BlockExecutionDBTransactionsMock(this.sinon);

    blockExecutionStateMock = new BlockExecutionStateMock(this.sinon);

    beginBlockHandler = beginBlockHandlerFactory(
      blockchainState,
      blockExecutionDBTransactionsMock,
      blockExecutionStateMock,
    );

    blockHeight = 2;

    request = {
      header: {
        height: blockHeight,
      },
    };
  });

  it('should update height, start transactions return ResponseBeginBlock', async () => {
    const response = await beginBlockHandler(request);

    expect(response).to.be.an.instanceOf(ResponseBeginBlock);

    expect(blockchainState.getLastBlockHeight()).to.equal(blockHeight);

    expect(blockExecutionDBTransactionsMock.start).to.be.calledOnce();
    expect(blockExecutionStateMock.reset).to.be.calledOnce();
  });
});