    #[test]
    fn de_clipper_clamps_large_signals() {
        let mut dcl = DeClipper::new();
        dcl.initialize(48000.0, 0.5);
        for _ in 0..16 {
            let out = dcl.process(2.0);
            assert!(out.is_finite());
            assert!(out.abs() <= 2.0);
        }
    }