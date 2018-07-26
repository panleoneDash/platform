const DockerInstanceOptions = require('../docker/DockerInstanceOptions');

class IPFSInstanceOptions extends DockerInstanceOptions {
  constructor() {
    super();

    const ipfsPort = this.getRandomPort(10001, 19998);
    this.ipfs = {
      exposedPort: ipfsPort,
      internalPort: 5001,
    };
    const container = {
      image: 'ipfs/go-ipfs:v0.4.15',
      network: {
        name: 'dash_test_network',
        driver: 'bridge',
      },
      entrypoint: [
        '/sbin/tini', '--',
        '/bin/sh', '-c',
        [
          'ipfs init',
          'ipfs config --json Bootstrap []',
          'ipfs config --json Discovery.MDNS.Enabled false',
          `ipfs config Addresses.API /ip4/0.0.0.0/tcp/${this.ipfs.internalPort}`,
          'ipfs config Addresses.Gateway /ip4/0.0.0.0/tcp/8080',
          'ipfs daemon',
        ].join(' && '),
      ],
      ports: [
        `${ipfsPort}:${this.ipfs.internalPort}`,
      ],
    };
    this.container = { ...this.container, ...container };
  }

  /**
   * Regenerate IPFS exposed port
   *
   * @returns {IPFSInstanceOptions}
   */
  regeneratePorts() {
    const ipfsPort = this.getRandomPort(10001, 19998);

    this.ipfs.exposedPort = ipfsPort;
    this.container.ports = [
      `${ipfsPort}:5001`,
    ];

    return this;
  }

  /**
   * Get IPFS exposed port
   *
   * @returns {number}
   */
  getIpfsExposedPort() {
    return this.ipfs.exposedPort;
  }

  /**
   * Get IPFS internal port
   *
   * @returns {number}
   */
  getIpfsInternalPort() {
    return this.ipfs.internalPort;
  }
}

module.exports = IPFSInstanceOptions;