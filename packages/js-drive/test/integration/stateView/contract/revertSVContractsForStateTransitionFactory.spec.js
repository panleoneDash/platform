const {
  mocha: {
    startMongoDb,
    startIPFS,
  },
} = require('@dashevo/dp-services-ctl');

const DashPlatformProtocol = require('@dashevo/dpp');

const ReaderMediator = require('../../../../lib/blockchain/reader/BlockchainReaderMediator');

const Revision = require('../../../../lib/stateView/revisions/Revision');
const Reference = require('../../../../lib/stateView/revisions/Reference');
const SVContract = require('../../../../lib/stateView/contract/SVContract');
const SVContractMongoDbRepository = require('../../../../lib/stateView/contract/SVContractMongoDbRepository');

const getBlocksFixture = require('../../../../lib/test/fixtures/getBlocksFixture');
const getSTPacketsFixture = require('../../../../lib/test/fixtures/getSTPacketsFixture');
const getStateTransitionsFixture = require('../../../../lib/test/fixtures/getStateTransitionsFixture');
const getSVContractFixture = require('../../../../lib/test/fixtures/getSVContractFixture');
const getReferenceFixture = require('../../../../lib/test/fixtures/getReferenceFixture');
const getDocumentsFixture = require('../../../../lib/test/fixtures/getDocumentsFixture');

const RpcClientMock = require('../../../../lib/test/mock/RpcClientMock');
const ReaderMediatorMock = require('../../../../lib/test/mock/BlockchainReaderMediatorMock');

const STPacketIpfsRepository = require('../../../../lib/storage/stPacket/STPacketIpfsRepository');
const updateSVContractFactory = require('../../../../lib/stateView/contract/updateSVContractFactory');
const revertSVContractsForStateTransitionFactory = require('../../../../lib/stateView/contract/revertSVContractsForStateTransitionFactory');
const applyStateTransitionFactory = require('../../../../lib/stateView/applyStateTransitionFactory');
const applyStateTransitionFromReferenceFactory = require('../../../../lib/stateView/applyStateTransitionFromReferenceFactory');

describe('revertSVContractsForStateTransitionFactory', () => {
  let stPacketRepository;
  let svContractMongoDbRepository;
  let applyStateTransition;
  let rpcClientMock;
  let readerMediator;
  let revertSVContractsForStateTransition;
  let mongoDatabase;
  let ipfsClient;
  let userId;

  startMongoDb().then((mongoDb) => {
    mongoDatabase = mongoDb.getDb();
  });

  startIPFS().then((ipfs) => {
    ipfsClient = ipfs.getApi();
  });

  beforeEach(function beforeEach() {
    ({ userId } = getDocumentsFixture);

    const dpp = new DashPlatformProtocol({
      dataProvider: {},
    });

    stPacketRepository = new STPacketIpfsRepository(
      ipfsClient,
      dpp,
      1000,
    );

    svContractMongoDbRepository = new SVContractMongoDbRepository(mongoDatabase, dpp);

    const updateSVContract = updateSVContractFactory(svContractMongoDbRepository);

    readerMediator = new ReaderMediatorMock(this.sinon);

    applyStateTransition = applyStateTransitionFactory(
      stPacketRepository,
      updateSVContract,
      null,
      readerMediator,
    );

    rpcClientMock = new RpcClientMock(this.sinon);

    const applyStateTransitionFromReference = applyStateTransitionFromReferenceFactory(
      applyStateTransition,
      rpcClientMock,
    );

    revertSVContractsForStateTransition = revertSVContractsForStateTransitionFactory(
      svContractMongoDbRepository,
      rpcClientMock,
      applyStateTransition,
      applyStateTransitionFromReference,
      readerMediator,
    );
  });

  it('should remove last version of SV Contract and re-apply previous versions in order', async () => {
    // 1. Store 3 versions of Contracts in IPFS
    const contractVersions = [];

    const blocks = getBlocksFixture();
    const stateTransitions = getStateTransitionsFixture();
    const [stPacket] = getSTPacketsFixture();

    const contractId = stPacket.getContractId();
    const contract = stPacket.getContract();

    for (let i = 0; i < 3; i++) {
      const block = blocks[i];
      const stateTransition = stateTransitions[i];

      contract.setVersion(i + 1);

      await stPacketRepository.store(stPacket);

      stateTransition.extraPayload.hashSTPacket = stPacket.hash();

      const reference = new Reference({
        blockHash: block.hash,
        blockHeight: block.height,
        stHash: stateTransition.hash,
        stPacketHash: stPacket.hash(),
        hash: contract.hash(),
      });

      contractVersions.push({
        version: (i + 1),
        block,
        stateTransition,
        stPacket,
        reference,
      });

      rpcClientMock.getRawTransaction
        .withArgs(stateTransition.hash)
        .resolves({
          result: stateTransition,
        });
    }

    // 2. Create ans store SV Contract
    const previousRevisions = contractVersions.slice(0, 2)
      .map(({ version, reference }) => (
        new Revision(version, reference)
      ));

    const svContract = new SVContract(
      contractId,
      userId,
      contract,
      contractVersions[contractVersions.length - 1].reference,
      false,
      previousRevisions,
    );

    await svContractMongoDbRepository.store(svContract);

    // 3. Revert 3rd version of contract to 2nd
    const thirdContractVersion = contractVersions[contractVersions.length - 1];

    await revertSVContractsForStateTransition({
      stateTransition: thirdContractVersion.stateTransition,
      block: thirdContractVersion.block,
    });

    const revertedSVContract = await svContractMongoDbRepository.find(contractId);

    expect(revertedSVContract.getContract().getVersion()).to.equal(2);

    expect(revertedSVContract.getPreviousRevisions()).to.deep.equal([
      previousRevisions[0],
    ]);

    expect(readerMediator.emitSerial.getCall(1)).to.have.been.calledWith(
      ReaderMediator.EVENTS.CONTRACT_REVERTED,
      {
        userId: thirdContractVersion.stateTransition.extraPayload.regTxId,
        contractId,
        reference: thirdContractVersion.reference,
        contract: contract.toJSON(),
        previousRevision: previousRevisions[previousRevisions.length - 1],
      },
    );
  });

  it('should delete SV Contract if there are no previous versions', async () => {
    const svContract = getSVContractFixture();
    const [stateTransition] = getStateTransitionsFixture();
    const [block] = getBlocksFixture();
    const reference = getReferenceFixture();

    const contractId = svContract.getContractId();

    await svContractMongoDbRepository.store(svContract);

    await revertSVContractsForStateTransition({
      stateTransition,
      block,
    });

    const revertedSVContract = await svContractMongoDbRepository.find(contractId);

    expect(revertedSVContract).to.not.exist();

    expect(readerMediator.emitSerial).to.have.been.calledWith(
      ReaderMediator.EVENTS.CONTRACT_MARKED_DELETED,
      {
        userId: stateTransition.extraPayload.regTxId,
        contractId,
        reference,
        contract: svContract.getContract().toJSON(),
      },
    );
  });
});