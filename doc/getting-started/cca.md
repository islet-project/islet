# ARM Confidential Compute Architecture (CCA)

[ARM CCA](https://www.arm.com/architecture/security-features/arm-confidential-compute-architecture) is the latest confidential computing technology that can extend confidential computing through to mobile devices (i.e., Samsung galaxy). For ARM-based devices, TrustZone has been the pillar of secure compute for over a decade and adopted for various use cases. However, one weakness of TrustZone makes it hard to keep up with an increasing number of applications that want to benefit from TrustZone. That is the lack of dynamic yet flexible memory allocation strategy.

To isolate TrustZone from normal worlds (non-secure worlds), hardware manufacturer like Samsung have had to dedicate some portion of physical memory to TrustZone,
which raises a conventional memory-related tradeoff. To be fair, it's not a problem that only belongs to TrustZone, some other TEEs (e.g., SGX) also suffer from it.
And this is one of the reasons why recent confidential computing architectures take secure virtual machine approach (e.g., AMD SEV, Intel TDX, ARM CCA) over process-based ones (e.g., Intel SGX).

From the hardware manufacturer perspective, the capability of dynamic secure memory allocation is definitely the most appealing feature among others.
But, this is not the only thing Islet is excited about. Islet can benefit from ARM CCA in many aspects that include but not limited to:

- dynamic secure memory allocation, which allows more secure applications to coexist with non-secure applications.
- attestation, which allows other entities (e.g., service provider) to easily verify applications running on mobile devices, which in turn making things easier to build complex trustworthy services.
- device protection, which could be accomplished by a so-called secure virtualization as specified in [this blog post](https://community.arm.com/arm-community-blogs/b/architectures-and-processors-blog/posts/introducing-arms-dynamic-trustzone-technology).

On top of the above features, what's interesting to Islet is that CCA leaves the role of implementing key components that act as TCB (Trusted Computing Base) to manufacturers. In other words, hardware vendors can augment CCA to solve their unique challenges as long as their implementations adhere to the CCA specification.
This flexibility would get significantly important considering updates when a new threat to confidential computing emerges.
For example, there had been a lot of side-channel attacks targeting Intel SGX. However, since Intel SGX puts all security-related codes in hardware, such attacks couldn't be mitigated by platform updates, demanding updates on a per-application basis.

We believe that Islet takes advantages of strong features of CCA while augmenting CCA in many aspects to get to the point where mobile device users truly get a great security experience.
