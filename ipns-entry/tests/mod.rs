mod auto_traits;

mod integration {
    use ipns_entry::cbor;
    use ipns_entry::entry::{IpnsEntry, ValidityType};
    use ipns_entry::signer::{V1Signer, V2Signer};
    use libp2p_identity::ed25519;
    use libp2p_identity::Keypair;
    use libp2p_identity::PeerId;
    use libp2p_identity::PublicKey;
    #[test]
    fn test_create_entry_pb_bytes() {
        let keypair = Keypair::generate_ed25519()
            .try_into_ed25519()
            .expect("A ed25519 keypair");

        let value = "QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq";
        let validity = "2033-05-18T03:33:20.000000000Z";
        let sequence = 0;
        let ttl = 0;

        let data = cbor::Data {
            value: value.as_bytes().to_vec(),
            validity: validity.as_bytes().to_vec(),
            validity_type: 0,
            sequence,
            ttl,
        }
        .to_bytes();

        let v2_signer = V2Signer::new(&keypair);
        let sig_v2 = v2_signer.sign(&data);

        // verify keypair ops
        assert!(v2_signer.verify(&data, &sig_v2));

        let sig_v1 = (V1Signer {
            keypair: keypair.clone(),
            validity: validity.as_bytes(),
            value: value.as_bytes(),
            validity_type: 0,
        })
        .sign();

        let entry = IpnsEntry {
            value: Some(value.as_bytes().into()),
            validity: Some(validity.as_bytes().into()),
            validity_type: Some(ValidityType::Eol.into()),
            signature_v1: Some(sig_v1.clone()),
            signature_v2: Some(sig_v2.clone()),
            sequence: Some(0),
            data: Some(data.clone()),
            ttl: Some(ttl),
            pub_key: Some(keypair.public().to_bytes().to_vec()),
        };

        let bytes = entry.to_bytes();

        // println!("{}", hex::encode(&bytes));

        // decode protobuf bytes back into struct
        let entry = IpnsEntry::from_bytes(&bytes).unwrap();

        assert_eq!(entry.value, Some(value.as_bytes().into()));
        assert_eq!(entry.validity, Some(validity.as_bytes().into()));
        assert_eq!(entry.validity_type, Some(ValidityType::Eol.into()));
        assert_eq!(entry.signature_v1, Some(sig_v1));
        assert_eq!(entry.signature_v2, Some(sig_v2));
        assert_eq!(entry.sequence, Some(0));
        assert_eq!(entry.data, Some(data));
        assert_eq!(entry.ttl, Some(ttl));
        assert_eq!(entry.pub_key, Some(keypair.public().to_bytes().to_vec()));

        // convert public key to binary_id
        // BINARY_ID is the binary representation of IPNS Name
        // IPNS Name is a Multihash of a serialized PublicKey
        // a Multihash of a serialized PublicKey is the same as the PeerId?
        let ed_key = ed25519::PublicKey::try_from_bytes(entry.pub_key.as_ref().unwrap()).unwrap();
        let pub_key: PublicKey = PublicKey::from(ed_key);

        // PeerId will create the multihash for us, to_byte returns the binary representation of that multihash
        let binary_peer_id = PeerId::from_public_key(&pub_key).to_bytes();

        // assert is_valid
        assert!(entry.is_valid_for(binary_peer_id).is_ok());
    }

    #[test]
    fn test_ipns_js_compat() {
        // These values were generated using the js-ipns library
        let validity = "323032332d30342d31375431373a33343a30342e3232393030303030305a";
        let sig_v1 = "305a44a40877a7dd5f4f6ef6d8b03b52c3f053c2f307af712c19f2944724610ba463df84cfc50ba68d524dc7eb28e90d6ffeba478c24bdedb1477aacc4dfe7c158c3de5f9f68a644e332af298461c4052db360e04b3e90486a5ecf5f19f6d1d4df83e330ecbcab608804e00e6951164367dfdc8d75fce35936d3f75260f291489ee68dffe12440552b11cff2022979ffddfe72f8492150aef8badec6fecbe46e1189d02b10c651ef852d2f1edf79fba7880a32cd79c350021f922872f7067a0631dde7eebd892886023d6f8aecb5bcf3ed6cd36860800aedd03ec653c183afccc0bda5dbb30e548e0743f65a64625f46d423d650b41c678ac0fabc82b710f146";
        let sig_v2 = "24ee21a4e266d337e029a5367d9598c00a29e284579e543d60c7da3bc34176570c865f82cec0b3e1b301acf969d6250333002b439ae5e746692e7bfa50ab19117e44139007b419a8db69b8bacd5b986e6dc0d8a8a345023c79bf02cb8ced9da5d83185ae970d6839fe485d2cc27db0d9df12bf6207f7fb9166acc1ddf444ed8d216838a235889b115d6d023204473e3f5a422c2a93d8620336d31f523cfb02b1f6b3762f30f4fcaa21dff2c9033c6de21e306ee936580dd56a6c99670ecb81305c4679475ecebe240df553d4d41886c2edb0399ebbf48020579300eb374ad7d9cc944ab5d8e7cf3c843c0f9b72c63bf3572ea2ae36a6672751d0ba4ab98871ff";
        let data = "a56354544c1b000000174876e8006556616c7565582e516d5745656b5837455a4c5564395658524e4d525857334c586534463678376d42386f50785935584c70747242716853657175656e6365006856616c6964697479581e323032332d30342d31375431373a33343a30342e3232393030303030305a6c56616c69646974795479706500";
        let pub_key = "080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100979f7b4845399afdacefefd57f15e3510011861a966174421c99ffef1a015b8ee9c4285e7c9f112a7b5d30c839b3455294f7fd526f02be45f1643abafa3f12675fa3040e63b38f9f7c6bdf340dc9046c82cc7b83e18361218952dc8bfc38be890cabcd8d3d5da4d21e4e17527c1f90548e263428cf62113375b6d1692c4a6232e417e8d5d83f9bba0ea108f8d23e3d17b3a7c395521d6b8c438668331ae1ba318ce55f0519a1e587748bef36999e69c65eb0bcc1f299d9e74e087c5d072bf9af781b03b43a0cef38c267f3a65143826e2b3be6f0c7841efd3c5c54683f8aed2c7028fbe65d3c261c80cc75be2a3872f7fd3856770ffe71d2687bcd4a91a34d1f0203010001";
        let pb_bytes = "0a2e516d5745656b5837455a4c5564395658524e4d525857334c586534463678376d42386f50785935584c7074724271128002305a44a40877a7dd5f4f6ef6d8b03b52c3f053c2f307af712c19f2944724610ba463df84cfc50ba68d524dc7eb28e90d6ffeba478c24bdedb1477aacc4dfe7c158c3de5f9f68a644e332af298461c4052db360e04b3e90486a5ecf5f19f6d1d4df83e330ecbcab608804e00e6951164367dfdc8d75fce35936d3f75260f291489ee68dffe12440552b11cff2022979ffddfe72f8492150aef8badec6fecbe46e1189d02b10c651ef852d2f1edf79fba7880a32cd79c350021f922872f7067a0631dde7eebd892886023d6f8aecb5bcf3ed6cd36860800aedd03ec653c183afccc0bda5dbb30e548e0743f65a64625f46d423d650b41c678ac0fabc82b710f1461800221e323032332d30342d31375431373a33343a30342e3232393030303030305a28003080d0dbc3f4023aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100979f7b4845399afdacefefd57f15e3510011861a966174421c99ffef1a015b8ee9c4285e7c9f112a7b5d30c839b3455294f7fd526f02be45f1643abafa3f12675fa3040e63b38f9f7c6bdf340dc9046c82cc7b83e18361218952dc8bfc38be890cabcd8d3d5da4d21e4e17527c1f90548e263428cf62113375b6d1692c4a6232e417e8d5d83f9bba0ea108f8d23e3d17b3a7c395521d6b8c438668331ae1ba318ce55f0519a1e587748bef36999e69c65eb0bcc1f299d9e74e087c5d072bf9af781b03b43a0cef38c267f3a65143826e2b3be6f0c7841efd3c5c54683f8aed2c7028fbe65d3c261c80cc75be2a3872f7fd3856770ffe71d2687bcd4a91a34d1f020301000142800224ee21a4e266d337e029a5367d9598c00a29e284579e543d60c7da3bc34176570c865f82cec0b3e1b301acf969d6250333002b439ae5e746692e7bfa50ab19117e44139007b419a8db69b8bacd5b986e6dc0d8a8a345023c79bf02cb8ced9da5d83185ae970d6839fe485d2cc27db0d9df12bf6207f7fb9166acc1ddf444ed8d216838a235889b115d6d023204473e3f5a422c2a93d8620336d31f523cfb02b1f6b3762f30f4fcaa21dff2c9033c6de21e306ee936580dd56a6c99670ecb81305c4679475ecebe240df553d4d41886c2edb0399ebbf48020579300eb374ad7d9cc944ab5d8e7cf3c843c0f9b72c63bf3572ea2ae36a6672751d0ba4ab98871ff4a8501a56354544c1b000000174876e8006556616c7565582e516d5745656b5837455a4c5564395658524e4d525857334c586534463678376d42386f50785935584c70747242716853657175656e6365006856616c6964697479581e323032332d30342d31375431373a33343a30342e3232393030303030305a6c56616c69646974795479706500";

        let mut entry = IpnsEntry {
            value: Some(b"QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq".to_vec()),
            validity: hex::decode(validity).ok(),
            validity_type: Some(ValidityType::Eol.into()),
            signature_v1: hex::decode(sig_v1).ok(),
            signature_v2: hex::decode(sig_v2).ok(),
            sequence: Some(0),
            data: hex::decode(data).ok(),
            pub_key: hex::decode(pub_key).ok(),
            ttl: Some(100000000000),
            // ..ipns::IpnsEntry::default()
        };

        entry.set_validity_type(ValidityType::Eol);

        let buf = entry.to_bytes();

        // buf should match pb_bytes
        assert_eq!(buf, hex::decode(pb_bytes).unwrap());

        let exit_bytes = IpnsEntry::from_bytes(&buf);
        assert_eq!(entry, exit_bytes.expect("failed to decode ipns entry"));
    }

    #[test]
    fn test_fails_on_bad_signature() {
        //make signature_v2 bogus, verify should fail
        let keypair = Keypair::generate_ed25519()
            .try_into_ed25519()
            .expect("A ed25519 keypair");

        let value = "QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq";
        let validity = "2033-05-18T03:33:20.000000000Z";
        let sequence = 0;
        let ttl = 0;

        let data = cbor::Data {
            value: value.as_bytes().to_vec(),
            validity: validity.as_bytes().to_vec(),
            validity_type: 0,
            sequence,
            ttl,
        }
        .to_bytes();

        let sig_v2 = hex::decode("deadbeef").unwrap();

        let sig_v1 = (V1Signer {
            keypair: keypair.clone(),
            validity: validity.as_bytes(),
            value: value.as_bytes(),
            validity_type: 0,
        })
        .sign();

        let entry = IpnsEntry {
            value: Some(value.as_bytes().into()),
            validity: Some(validity.as_bytes().into()),
            validity_type: Some(ValidityType::Eol.into()),
            signature_v1: Some(sig_v1),
            signature_v2: Some(sig_v2),
            sequence: Some(0),
            data: Some(data),
            ttl: Some(ttl),
            pub_key: Some(keypair.public().to_bytes().to_vec()),
        };

        let ed_key = ed25519::PublicKey::try_from_bytes(entry.pub_key.as_ref().unwrap()).unwrap();
        let pub_key: PublicKey = PublicKey::from(ed_key);

        // PeerId will create the multihash for us, to_byte returns the binary representation of that multihash
        let binary_peer_id = PeerId::from_public_key(&pub_key).to_bytes();

        assert!(!entry.is_valid_for(binary_peer_id).expect("an answer"));
    }
}
