use crate::{affine::AffinePoint, field::Field, modulus::EllipticCurve};
use num_traits::Zero;
use rand::Rng;
use std::ops::{Add, AddAssign, Mul};

#[derive(Debug, PartialEq, PartialOrd, Eq, Default)]
pub struct ProjectivePoint<M: EllipticCurve> {
    pub x: Field<M>,
    pub y: Field<M>,
    pub z: Field<M>,
}

impl<M: EllipticCurve> ProjectivePoint<M> {
    pub fn new_infinity() -> Self {
        Self {
            x: Field::<M>::from(0),
            y: Field::<M>::from(1),
            z: Field::<M>::from(0),
        }
    }

    pub fn is_infinity(&self) -> bool {
        self.x.number.is_zero() && self.y == Field::<M>::from(1) && self.z.number.is_zero()
    }

    pub fn double(&self) -> Self {
        if self.is_infinity() {
            return Self::new_infinity();
        }

        if self.y.number.is_zero() {
            return Self::new_infinity();
        }

        let a = Field::<M>::from(M::a());
        let two = Field::<M>::from(2);
        let three = Field::<M>::from(3);
        let four = Field::<M>::from(4);
        let eight = Field::<M>::from(8);

        let z_sq = &self.z * &self.z;
        let x_sq = &self.x * &self.x;
        let y_sq = &self.y * &self.y;

        // W := a*Z^2 + 3*X^2
        let w = &(&a * &z_sq) + &(&three * &x_sq);

        // S := Y*Z
        let s = &self.y * &self.z;
        let s_sq = &s * &s;

        // B := X*Y*S
        let b = &self.x * &self.y * &s;

        // H := W^2 - 8*B
        let w_sq = &w * &w;
        let h = &w_sq - &(&eight * &b);

        // X' := 2*H*S
        let x_prime = &two * &h * &s;

        // Y' := W*(4*B - H) - 8*Y^2*S^2
        let four_b_minus_h = &(&four * &b) - &h;
        let eight_y2_s2 = &eight * &(&y_sq * &s_sq);
        let y_prime = &(&w * &four_b_minus_h) - &eight_y2_s2;

        // Z' := 8*S^3
        let s_cube = &s_sq * &s;
        let z_prime = &eight * &s_cube;

        Self {
            x: x_prime,
            y: y_prime,
            z: z_prime,
        }
    }

    pub fn add_points(&self, rhs: &Self) -> Self {
        if self.is_infinity() {
            return rhs.clone();
        } else if rhs.is_infinity() {
            return self.clone();
        }

        // U1 := Y2*Z1
        let u1 = &rhs.y * &self.z;
        // U2 := Y1*Z2
        let u2 = &self.y * &rhs.z;
        // V1 := X2*Z1
        let v1 = &rhs.x * &self.z;
        // V2 := X1*Z2
        let v2 = &self.x * &rhs.z;

        if v1 == v2 {
            if u1 != u2 {
                return Self::new_infinity();
            } else {
                return self.double();
            }
        }

        let two = Field::<M>::from(2);

        // U := U1 - U2
        let u = &u1 - &u2;
        let u_sq = &u * &u;

        // V := V1 - V2
        let v = &v1 - &v2;
        let v_sq = &v * &v;
        let v_cube = &v_sq * &v;

        // W := Z1*Z2
        let w = &self.z * &rhs.z;

        // A := U^2*W - V^3 - 2*V^2*V2
        let u2_w = &u_sq * &w;
        let two_v2_v2 = &two * &(&v_sq * &v2);
        let a = &(&u2_w - &v_cube) - &two_v2_v2;

        // X3 := V*A
        let x3 = &v * &a;

        // Y3 := U*(V^2*V2 - A) - V^3*U2
        let v2_v2_minus_a = &(&v_sq * &v2) - &a;
        let v3_u2 = &v_cube * &u2;
        let y3 = &(&u * &v2_v2_minus_a) - &v3_u2;

        // Z3 := V^3*W
        let z3 = &v_cube * &w;

        Self {
            x: x3,
            y: y3,
            z: z3,
        }
    }

    pub fn mul_scalar(&self, k: &Field<M>) -> Self {
        let mut r0 = Self::new_infinity();
        let mut r1 = self.clone();

        let bits = k.number.bits();

        if bits == 0 {
            return r0;
        }

        for i in (0..bits).rev() {
            if k.number.bit(i) {
                r0 = r0.add_points(&r1);
                r1 = r1.double();
            } else {
                r1 = r0.add_points(&r1);
                r0 = r0.double();
            }
        }

        r0
    }

    fn get_generator() -> Self {
        Self {
            x: Field::<M>::new(M::gen_x().clone()),
            y: Field::<M>::new(M::gen_y().clone()),
            z: Field::<M>::one(),
        }
    }

    pub fn get_random<R: Rng + Sized>(rng: &mut R) -> Self {
        Self::get_generator() * Field::<M>::get_random(rng)
    }
}

impl<M: EllipticCurve> Add for &ProjectivePoint<M> {
    type Output = ProjectivePoint<M>;

    fn add(self, rhs: Self) -> Self::Output {
        self.add_points(rhs)
    }
}

impl<M: EllipticCurve> Add for ProjectivePoint<M> {
    type Output = ProjectivePoint<M>;

    fn add(self, rhs: Self) -> Self::Output {
        self.add_points(&rhs)
    }
}

impl<M: EllipticCurve> AddAssign for ProjectivePoint<M> {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add_points(&rhs);
    }
}

impl<M: EllipticCurve> Mul<&Field<M>> for &ProjectivePoint<M> {
    type Output = ProjectivePoint<M>;

    fn mul(self, rhs: &Field<M>) -> Self::Output {
        let mut r0 = ProjectivePoint::new_infinity();
        let mut r1 = self.clone();

        let bits = rhs.number.bits();

        if bits == 0 {
            return r0;
        }

        for i in (0..bits).rev() {
            if rhs.number.bit(i) {
                r0 = r0.add_points(&r1);
                r1 = r1.double();
            } else {
                r1 = r0.add_points(&r1);
                r0 = r0.double();
            }
        }

        r0
    }
}

impl<M: EllipticCurve> Mul<Field<M>> for ProjectivePoint<M> {
    type Output = ProjectivePoint<M>;

    fn mul(self, rhs: Field<M>) -> Self::Output {
        &self * &rhs
    }
}

impl<M> Clone for ProjectivePoint<M>
where
    M: EllipticCurve,
{
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.x.clone_from(&source.x);
        self.y.clone_from(&source.y);
        self.z.clone_from(&source.z);
    }
}

impl<M: EllipticCurve> From<ProjectivePoint<M>> for AffinePoint<M> {
    fn from(proj: ProjectivePoint<M>) -> Self {
        if proj.is_infinity() {
            return AffinePoint::new_infinity();
        }

        let z_inv = proj.z.inv().expect("Z coordinate must be invertible");
        let affine_x = &proj.x * &z_inv;
        let affine_y = &proj.y * &z_inv;
        AffinePoint {
            x: affine_x,
            y: affine_y,
            is_infinity: false,
        }
    }
}

impl<M: EllipticCurve> From<AffinePoint<M>> for ProjectivePoint<M> {
    fn from(affine: AffinePoint<M>) -> Self {
        if affine.is_infinity {
            return ProjectivePoint::new_infinity();
        }

        ProjectivePoint {
            x: affine.x.clone(),
            y: affine.y.clone(),
            z: Field::<M>::one(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::modulus::P521;

    use super::*;

    pub const ADD_TESTS: &[(&str, &str, &str, &str, &str, &str)] = &[
        // (P1_X, P1_Y, P2_X, P2_Y, EXP_X, EXP_Y)
        (
            "1a982fa66c848b655abd05a5d9edda8e35275c89a8138360e4ea11b9e9dd5830cc9bed1af4d4dd21f318528c74ea28b8bf8eab0f9021f2fddfcbadd3c2c3e474884",
            "9770a71a37845e478352d92403c772acd46135cbcf72723091157c67bcfdd15d9e0835262f05ece1721f7a27cc839f0a6360f60a9fdb426e1080186d6cb584deb8",
            "11e8221217a3bc36583780a6147508412c26490835d8f942697c07abd9aef1978689ab8cba5e3ae25f75da103271bdf481d903a1e43841e2f9f2714b1067620686b",
            "3674cfd6ae6220fae12310a720175ecd5f3a1e8febc19482fdf2386683ed1fc34e0869a312130f27e4807ba62fea5edab892b4e1ca745fea558e2224ee4343cb0e",
            "1ce8484ba335cc97ce755b34f54581ecc59485ddf362012dfae493980cb9d0c97e0c0f39a902a5ececee1f053c6090ef9d72e7521b62b168ef7a2caf27c49a0ab07",
            "139af3988e1eaf15e43faf8b88065fc260df1c57193beb77b1898a5ad8e1ef3a15b4d3cb091818803e817d28a7429f83515220961c4a273475d6c585b4dd19bb37a",
        ),
        (
            "1556cc26eabca4e7c5d5802544dc3343b9c56c76918e4cc47dc23061d9b8bc02056cccb026ecbc7841d07ac310ad6077d92f42aab22d2167e49a4a7575c1a9d02fd",
            "d05abd29371769241f19ab76a54a0be5a8f13210dfbc0b63106606613422d1022f6cabd07e69c806dc73ffb278b1fbcf37e50b16960ad53067912798a198d12e1",
            "204d5afa3a7bb2873da44241649c903bde432facec3e1df9cf85975fdb01ba0a19b685c5d8b2d33219e7616c55a4ab1ce597cbc63b9bb51089f77db8d27d9fb12d",
            "429ff42d847d09170eab6db3b98c43707a4349fbdc0ddc473914caaf74e7ab5a3aba80b9a4c250883704243cdf30b473dc34c27b3ce3de64ff37468ca425e656fb",
            "b4ab686f33d4d1b05eda8be8cc3d6c832616c6bdb54246dc807d8828dd0a7b040e872d18c0f487dc188fe2683bb0ba21e141e992693d006921ff1be577cbcb34c5",
            "6841ec395cbf839550d7d72bae1f65e2dcd16cb9da5c77ecd389d22e65c11111fb0979a64f45b2816053b859f10ca6788d44f5bc2fe6d2dfe0e6fea0a90953491a",
        ),
        (
            "1492795e772bd5a954380d9653fb4174eeee04092ba34b0e18c0d54340e14afb1852cad1550e87a5a1bfa70635ba65f59846b69e22846ed685908c70bac456a1612",
            "80d0817f6f424d908fe9f3990e3b86bd9d9ae9df6e8c108620e68111d481797273cc0bdc0a31441d76633f1bb2086150282d513a8ecb02d091ce56e1d969f929c9",
            "9a5329ec46ff81f006d33698aac9b5fb894099c9f6e1cde8b8effe0b202fe62692a99ecff653ea91a9f9076d97e71650865a39f5ec82e76118ad0984dac57fdcc6",
            "1b68f2fab1caf237e266dfc4f061f4069868b567e83db9203f3e995efc88fae24cd6f9fa7c0391cc6453aa3eed6c4dd1a60d2ba59ced58e939922abb0416d47b020",
            "505426142b37241b5f62ab8ef974ebd410ce1863ab2d7bba2b80b6a5408e32c4eeb16d4cc952f377130bd270248bf97a8bb0c82a17ac04798f38ac3408b098e6f1",
            "14580b9cfcf56df75da0f9e3729bf1ca5c974bb0755892c7d3a3f98d7e53dac975a80a3cb0e343337dba8a78efd26346d9f4d024f2ca3a57048e034f6d81e6f04f9",
        ),
        (
            "1dbaaf46031214154e2912264da8c23bb5fcd94de5616f7305a1791a7297b9914460822639c6feba5fd096ff18335bdf52b8a3a00e8edcf8c023edc329aa0831b82",
            "ad11a7b3617af13c4d9c152d1c09943945a48a72cfb0fbfc54fe9d378d022eb5a0bd7389e904a6331f4d4b3438046fea5f6ab36956229a0dfec8b83bd3d38969ec",
            "18470ce42b9c5d49542005736b8df6864afc05d24792eb65783801853f0e1d6460f71388c5e4d6b4f44de8236603bcc5a7147a22ec1ed02d98e4f7d5f2a40eb9bbc",
            "37192523d2d193cb7e531c53b9a4d3ac22816e92b18197565a9fa9c2664918e845ae1260674c6d4b3e6c2306162996de930d815f407b68d5d2d1f8d3bdd6e588c6",
            "129f8cf08d0e0fddf86bdb1b12fe9fc37f8ce50811ede506cd373a5cf6f0a4ed978d10fbba2d933f777c7e08ffe4c9a21fbecadfbc727ccb536b594044085ae8f27",
            "9d8298dfccdbd232593c3d5d7765c116b5a2ded30df7063448a451088fa3876a50b9e9f6d77e447144ce093ae925050151e0229e2a9b1ce90f4577c5abd7720c07",
        ),
        (
            "1c37ce100ff1008c8240cfdc0510e5bad119d9e90334ae075f99b159e8e5034dbaded7ad541649a29d194a6afafb1947b553dec57fd8d0899926595bed5a9726295",
            "76ab11ad8ac90875dfc3433f98cab8a1a103ac9720f4b0bfcecb44518d84333e1727771320c24b4106527181daf08412706e7b4a16aedddcca2da5fc85bbe240a7",
            "a98fad385acb1005786dc655a09063a8201626b5ddd48bb762614a0d8d139a694a66ef02f4ac6824433400f2448f22c547a528b08ba7215c7fdf14edbc328dcf3d",
            "1083c1ca689a9c7870c214ae7b719fd76e6f07020847b829580445e7d4a0f704a87797721f59750df8a6416438c3a235960536cc5873a7a44e79d08a8a6cc116709",
            "dd1dcbcb6eca20aea957ab471a3aaa4ea578d4543921f2b4ed726860d784fcfe34955f733da7ae833dfaeb9177a5abab50ed5d5e6f8cd5f7701bc9a439ba58763d",
            "1d46b446bd0eebe37d1736a3172cc51c21027847c2833de6b15cebb4d3b97180e3b4b6291361bfd9b1d9e436a10d7fcead5e63ad1ec3b6035fc26c286976d0362f3",
        ),
    ];

    pub const DOUBLE_TESTS: &[(&str, &str, &str, &str)] = &[
        // (P1_X, P1_Y, EXP_X, EPX_Y)
        (
            "1681312ac47959c35b72b0cfa9630f1ac98729cb12a54d3211a94c9b93ad45a7ad052d0f5e148d05f7bb41e02146d104dc261f27b65b3969fc7564878285004ff",
            "17a438b094976d054c2eae6ceea1ed10f360cf82af050b006f0d6fb7098d522cc310c4a04047d10aabaf51eca7b039108c21b86dd931414ff5a5234b303c6334ab2",
            "135b5ed1a63d39ca3f4a3398894094fbfa7cc1e3f667ed7f420e9ade91a12668f68ee37f96542fed40e2fe78fbab14ca259a45075294ca8df7e57b8ae723585b424",
            "1708c20643c0cebeb01cff639e0bc764545185c9d58f4fe7a0cf4aa71b79e09e85117db5e8781dde5f317207677e92c4f04bb830bea51ea261c2a2d9e2f078a2445",
        ),
        (
            "1a21606c711b0825c119481737549c0f177c504bc3ff2b045bdf5eef2b99adce22309e9c49d860733449755853cf9c808174e7ede44e23691952be586aeb90db621",
            "19813e7488cea773de53e3d0a60091b3251f371b5a49e241d1d19d7e7e0d7eafddb7a3a59b3fa488278012773c39968dd6dbb55f9b1b42055ca1ea2a91ea6f240a4",
            "12b712e2bfd4a3c0732b3f5cde2db737492017bdd9d0ca6cfa81ded46d00979f76efe2c7aaa5b95dec7e301e2f4e8090aed68a9223cde56714b95417a3002ac8ebd",
            "176cfb4b63056c52ee9d7ed83b12767a408f1332c7eb3505897e5950a705e59e5ef16f16649c486ad3acd71a978e2c743f03d51483893eb85635f7ed88b1f945636",
        ),
        (
            "1d28e40e694ba4d41efc94622d9c34ed9b98e29d4f3b64876513e5e9f8137a672394e6035bd648e93cf6519df65958129bd477d1535ea317da3015385c80ae49953",
            "5f8ccfefb49207160cd117d8a2c78af43a003d5083751b1baff6d101a9e4ad594973c225fb730ae603dc27597ec7627c2edfc9709e1c7b53eccf089788da1cb4a8",
            "1dd849a128dd8f889764c3d1fd005ecd514a8a60b278874be0e9d163069b9ae21547b6249241d218e69dcc9f49f1ae5872802a923d3d4ad0d865bf888df89f443a7",
            "18da1e19eca36e853b03d41c7412a79c48a093066b85d821d38e52f0b21a59a8ac0516ce894591018d0815a7c3791884909186716b27bcd3a4edd9321ddcd2e7d8e",
        ),
        (
            "454781229127f6a83bb5d206265fa40122174be656b8eb3f386a8b478cad6c77842e285c1213cdfdc2f2ae48c76ecea3fc19e87355de8ce1c0c9911a2c274d1fce",
            "1e0188bd82e94a5fce141c2ed263740fcd099b7659c126ac0e5624497865fb9bc43d1bdf3f628949fd005d42016b48e7f6ac3a02c7583df1a5522a6189f69f87183",
            "6ef8dfce85039ccf30af30aae38cf9d928e00cc33ee3c9527ff854f5d8b76a9bda442c96aee62bad82cbf9bd74bd6915663754c45b2f7ecf1098f2c4fbb4f72687",
            "1bc2c431d98de47c8647bfa74cd80f6c054fbe83d4147e1daa6a2da15cb2b37ac37fe779c1fb799d35df361fa91dc14339ae2e27b27f8fac821ed29d2bb414a6586",
        ),
        (
            "11823c904fcac3c07db1124a88c9c535949d0eacc8f4d3c18d02568f6718b87cf53dc34c561d61e2e8dd467a045504457254d3157fd36c09414dfa16eada0c376d3",
            "194299c962e60658d9f3f0f0162359556223b6d6af95dc52a334aefc740aaa7b30a6733a0b5e27c71b54b18acbaa482c560d81a56131391ee2007a37dfc5153667b",
            "19e9fe7c4835811da92266a8615ed4d553e521a52a3a963cdbc39e8a1f2e2de952a1c4f51828f1c0aa7a9fbdd948b5c86f5a466b1d17d260c4031c91817037155e8",
            "143fbc94ba8f8f000d6f082b1db3e6235a75fce453c9e4a51c1ecc7e4eeb96d905726f948cfed08878de3356d0246768b5a76b913a2f667aca5034459096f81c49b",
        ),
    ];

    pub const MUL_TESTS: &[(&str, &str, &str, &str, &str)] = &[
        // (P1_X, P1_Y, SCALAR, EXP_X, EPX_Y)
        (
            "4df6e930b23d133444d5ca6154a68459ebe8c68b4c944facdb7d012b924e64cfc9493818ecb88ac92a7fd6a8e55ca2116e9f3fcb4a2accfb09e4354017c227292d",
            "157f16244a7ae244819f39266c5032f0789f69f64e62d3deda02174582223ca4bd8ec2d0e7722a12991916c0c2dc94def6e1b9660e9cb9b04f40fab987b730ed8da",
            "2f112bcbb92438ba6a7a8548b00dff6c18c5bcd909355876404466013910008088b77d08bf56436d0e1611500481868d263aa07c92c5654445617ae3acd2bf5385",
            "11a4f2bc55525e2ee7f41987713e66cbcd9b680d5ad91dbc5dee503b614131d8a987853d5b597c680c0e663a14dd9f53c911987805a023ed7e3125e3b477703113a",
            "2631dc0f8cbd01acdee85d3c45aadc97b474014e6d0ca082707b61bc7bf9cf5b8bb28cad6a88fe6c4247b46534f65eaae8d6d263cc166fead5cd6ae0197cd562da",
        ),
        (
            "e41d2e716e8810658874e6d5ee51b64e616299b4560e11bdcdfcb9fd3233390f4ac49db6fffcd59f4b428dbb69dabf3c3ff3ac2a33d708eb4f5f9e83cb75083ba1",
            "81b36481f2c1f11397207f1a4089b7fe599cf455e06700b0192431ec21e1bb8821989b841d2f0c3595c9ab9cef202b7b834975aa442b41415839bf9f49e9761d5f",
            "174dd8a1ea7e9130e8c2f29a7dd2bf562bdc90874c7f93baa7f9aa7476c8c885c59306f16fd928723a47421f070a8b619614c8fb20367f177234751b1338a69b71a",
            "115ddc6ae23bb67d0cdafe8b124118651ee45f9c9a02548e90d4750718259a2e2bff7b3c0ac06428559db17adf7cf0945ac633ea21588ee75b561907b586752e505",
            "898c28355aebdb89f7281dc15666d1770c5ef74b01e25878342a89ae0c2161ad0fb6c7143356bbb783e61fef0052e940d89a758eef8b70517da8d1fee8359ed5db",
        ),
        (
            "ce237bf3044112980cd09f23ef5faa6184ee8c4a1c8672e9c7b341d52df079dbbceda931717f72599ba8c9987c45b6d4528e0708a2ae541a0c18de87e42ef51872",
            "10bc0ff1f929446d2f3aa677a56c6d1aed38a3f39e7f59a231ce65d889e7fb39569a58ca141e5cfd291d2ebe46db4371d2bd14770cd7c10da1254d9fc1be1824195",
            "1479af48f54849d82bd6e4a907e422cc065be1a9d43ae48862de30e53984ee7dd7d87fc0f86e94621b8be3d41df4865d80d093a902dcc4b3236abc068309044ff31",
            "120a7d7d65e79e8beb2c783106bef203430eef1cff05449ec68327bc952e331a551540ef5a1cf9fdf3acc40f297cb5b582d012ff73bea6e616351e77f9762ba8a48",
            "1784ca8edf4830d3c6c7d69657b12e79235c327bbdea714c565af8a7ab52ca93eda67a2a22eb1f2d096bcf32c1c068a81d655f15e2736e88273e71607e9c6db28a1",
        ),
        (
            "17e272d6b146319b87f16c196bdbd2019a0f59a6d3af742e20dacdfb5eaa1a96903a58180a59cad572acc09a18891eb541186dce6a545c6721b8b729b58f4f0307b",
            "1d9f0cc7fcce4f64ca79c4d3bf25949f8a597052e765f543d5e74df602e4b30dc98a3441437dc9fbcf72116d93bf7ff2f9af033f6d9afe33a8dd7846b09a2107afd",
            "4a01ebfd363ef5af35b4ee3b6ebad36b3ec9be57682122634e29854fa1e5b4f8a1560bcf8ed384280f333ae654df91919c10ab6e6b23343e9d5f0779281d39dccc",
            "1a231f787f21ef6605199daf8a669dfbdd0b70a8395b8399fcdc1fbd17d9abe26e61a94522b4e9c903e612b69156a114b09d06c8d0f291fabd7416386238750da84",
            "191d6eee69d910781fbde875b84acb544d1a8901d279c909e1425af042a0d51aef408d7ccebadb941cbfe14dfce3972c5934789a30e27a1c97944ade4e689157ab1",
        ),
        (
            "5c776ec0a0c1f00935ca3e8bf07c90a41f0b39e749940185abded589a08455b4d308286ed6145a4763a800237e2c5a04e1eaed16c7a87ccbbfe2452d3ba6f56c39",
            "11f0030c5fc74cca0a2f4431ecd71775e9f2696e0b9b38838fed19366d75ecb7442ac75aca2157317fa9ba80319adffb377350e3c7914f95357e8c835fae1ef0939",
            "12fe69ed3b4ba5684ecd5e4282df870224bf751dac0729d5c1936bb15223318d8e6d7126d4c0055fb6f201b5ebd617964da971da0729f71392996974cc0f443287d",
            "1c9823c2d7f401c6e5aca77774f35667baa5ce8055a9e07263bac3f079285af099870ee114bc12bd3552854f99ccae41314414325df83966dd212d7b7044dfd3439",
            "16643edeb04263c2a7208114c1f2cd9ef25ac2777513b973bb886ee0a7cb38eecdfdf388780b655a7642e7f93a06ba0e8dd4e0ad809ebc6efffd92f86df04aaeba3",
        ),
    ];
    #[test]
    fn test_addition_roundtrip() {
        for (p1x, p1y, p2x, p2y, exp_x, exp_y) in ADD_TESTS {
            let p1_affine = AffinePoint::<P521>::from_hex(p1x, p1y).unwrap();
            let p2_affine = AffinePoint::<P521>::from_hex(p2x, p2y).unwrap();
            let expected_affine = AffinePoint::<P521>::from_hex(exp_x, exp_y).unwrap();

            let p1_proj: ProjectivePoint<P521> = p1_affine.into();
            let p2_proj: ProjectivePoint<P521> = p2_affine.into();

            let result_proj = p1_proj.add_points(&p2_proj);

            let result_affine: AffinePoint<P521> = result_proj.into();

            assert_eq!(result_affine.x.number, expected_affine.x.number);
            assert_eq!(result_affine.y.number, expected_affine.y.number);
        }
    }

    #[test]
    fn test_doubling_roundtrip() {
        for (p1x, p1y, exp_x, exp_y) in DOUBLE_TESTS {
            let p1_affine = AffinePoint::<P521>::from_hex(p1x, p1y).unwrap();
            let expected_affine = AffinePoint::<P521>::from_hex(exp_x, exp_y).unwrap();

            let p1_proj: ProjectivePoint<P521> = p1_affine.into();
            let result_proj = p1_proj.double();
            let result_affine: AffinePoint<P521> = result_proj.into();

            assert_eq!(result_affine.x.number, expected_affine.x.number);
            assert_eq!(result_affine.y.number, expected_affine.y.number);
        }
    }

    #[test]
    fn test_scalar_mul_roundtrip() {
        for (p1x, p1y, k_hex, exp_x, exp_y) in MUL_TESTS {
            let p1_affine = AffinePoint::<P521>::from_hex(p1x, p1y).unwrap();
            let k_field = Field::<P521>::from_hex(k_hex).unwrap();
            let expected_affine = AffinePoint::<P521>::from_hex(exp_x, exp_y).unwrap();

            let p1_proj: ProjectivePoint<P521> = p1_affine.into();
            let result_proj = p1_proj.mul_scalar(&k_field);
            let result_affine: AffinePoint<P521> = result_proj.into();

            assert_eq!(result_affine.x.number, expected_affine.x.number);
            assert_eq!(result_affine.y.number, expected_affine.y.number);
        }
    }
}
